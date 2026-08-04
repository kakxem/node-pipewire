use crate::{MainOptions, PipewireData, PipewireOptions, ALL_DATA};
use pipewire::{
    context::ContextRc,
    core::Core,
    link::Link,
    main_loop::MainLoopRc,
    node::Node,
    properties::properties,
    proxy::{Proxy, ProxyT},
    registry::{GlobalObject, Registry},
    spa::utils::dict::DictRef,
    types::ObjectType,
};

use std::{
    cell::RefCell,
    sync::{mpsc, Arc, Mutex},
};

thread_local! {
    static ENABLE_DEBUG: RefCell<bool> = RefCell::new(false);
}

pub(super) fn pw_thread(
    front_sender: mpsc::Sender<MainOptions>,
    pw_receiver: pipewire::channel::Receiver<PipewireOptions>,
    enable_debug: bool,
) {
    // Proxy cache to prevent destruction of elements while thread is running
    let proxies: Arc<Mutex<Vec<Proxy>>> = Arc::new(Mutex::new(Vec::new()));

    // Basic setup of pipewire thread
    let mainloop = MainLoopRc::new(None).expect("ERROR: error at creating mainloop");
    let context = ContextRc::new(&mainloop, None).expect("ERROR: error at creating context");
    let core = context
        .connect_rc(None)
        .expect("ERROR: error at connecting context");

    let registry = core
        .get_registry_rc()
        .expect("ERROR: error at getting registry");

    // Listen the pw_receiver the options from "PipewireOptions" struct
    let _receiver = pw_receiver.attach(&mainloop.loop_(), {
        let mainloop = mainloop.clone();
        let registry = core
            .get_registry_rc()
            .expect("ERROR: error at getting registry");

        let proxies = proxies.clone();

        move |msg| match msg {
            PipewireOptions::CloseThread => {
                if enable_debug {
                    println!("Closing pipewire thread");
                }
                mainloop.quit();
            }
            PipewireOptions::LinkNodesNameToId {
                output_nodes_name,
                input_node_id,
                permanent,
            } => {
                if enable_debug {
                    println!(
                        "Linking nodes: {:?} -> {:?}",
                        output_nodes_name, input_node_id
                    );
                }
                let links =
                    link_nodes_name_to_id(output_nodes_name, input_node_id, permanent, &core);

                let mut p = proxies.lock().unwrap();
                for link in links {
                    p.push(link.upcast());
                }
            }
            PipewireOptions::LinkPorts {
                input_port,
                output_port,
                permanent,
            } => {
                if enable_debug {
                    println!("Linking ports: {:?} -> {:?}", input_port, output_port);
                }
                let link = link_ports(input_port, output_port, permanent, &core);
                let mut p = proxies.lock().unwrap();
                p.push(link.upcast());
            }
            PipewireOptions::UnLinkNodesNameToId {
                output_nodes_name,
                input_node_id,
            } => {
                if enable_debug {
                    println!(
                        "Unlinking nodes: {:?} -> {:?}",
                        output_nodes_name, input_node_id
                    );
                }
                unlink_nodes_name_to_id(output_nodes_name, input_node_id, &registry)
            }
            PipewireOptions::UnLinkPorts {
                input_port,
                output_port,
            } => {
                if enable_debug {
                    println!("Unlinking ports: {:?} -> {:?}", input_port, output_port);
                }
                unlink_ports(input_port, output_port, &registry);
            }
            PipewireOptions::CreateSource {
                source_name,
                audio_position,
                channel_count,
                permanent,
            } => {
                if enable_debug {
                    println!(
                        "Creating virtual source named {:?} with position {:?}",
                        source_name, audio_position
                    );
                }
                let source =
                    create_source(source_name, audio_position, channel_count, permanent, &core);
                let mut p = proxies.lock().unwrap();
                p.push(source.upcast());
            }
            PipewireOptions::CreateSink {
                sink_name,
                audio_position,
                channel_count,
                permanent,
            } => {
                if enable_debug {
                    println!(
                        "Creating virtual sink named {:?} with position {:?}",
                        sink_name, audio_position
                    );
                }
                let sink = create_sink(sink_name, audio_position, channel_count, permanent, &core);
                let mut p = proxies.lock().unwrap();
                p.push(sink.upcast());
            }
            PipewireOptions::DeleteObject { id } => {
                if enable_debug {
                    println!("Attempting to destroy object {:?}", id);
                }
                destroy_object(id, &registry);
            }
        }
    });

    // Listen the registry for every change in the pipewire server.
    // The .global() method returns a "GlobalObject" struct.
    // The global_remove() method returns an id of the node/port/link that was removed.
    let _listener = registry
        .add_listener_local()
        .global({
            let sender = front_sender.clone();

            move |object| match object.type_ {
                ObjectType::Node => save_node(object, &sender),
                ObjectType::Port => save_port(object, &sender),
                ObjectType::Link => save_link(object, &sender),
                ObjectType::Client => save_client(object, &sender),
                _ => {
                    // Ignore other types.
                }
            }
        })
        .global_remove({
            let sender = front_sender.clone();
            let proxies = proxies.clone();

            move |id| {
                let mut p = proxies.lock().unwrap();
                if let Some(proxy) = p.iter().position(|proxy| proxy.id() == id) {
                    p.swap_remove(proxy);
                }
                sender
                    .send(MainOptions::DeleteItem { id })
                    .expect("ERROR: error at sending delete to front");
            }
        })
        .register();

    // save the enable_debug value in the thread local variable
    ENABLE_DEBUG.with(|e| *e.borrow_mut() = enable_debug);

    mainloop.run();
}

// Create a node and send it to the front.
fn save_node(node: &GlobalObject<&DictRef>, sender: &mpsc::Sender<MainOptions>) {
    // println!("Node: {:?}", node);

    let id = node.id;
    let permissions = node.permissions;
    let props = node
        .props
        .as_ref()
        .expect("ERROR: error at getting node properties");

    // create a hashmap that will contain all the properties of the node
    let mut node_props = std::collections::HashMap::new();

    // iterate over the properties and add them to the vector
    for (key, value) in props.iter() {
        node_props.insert(key.to_string(), value.to_string());
    }

    //  Get the nick name of the node. If is not found get the name of the node.
    let name = String::from(
        props
            .get("node.nick")
            .or_else(|| props.get("node.name"))
            .unwrap_or_default(),
    );

    // Get the type (Audio, Video, etc) of the node.
    // TODO: Search more types in documentation.
    let node_type = props
        .get("media.class")
        .and_then(|string| {
            let string = String::from(string);
            if string.contains("Audio") {
                Some("Audio")
            } else if string.contains("Video") {
                Some("Video")
            } else if string.contains("Midi") {
                Some("Midi")
            } else {
                None
            }
        })
        .unwrap_or_default()
        .to_string();

    // Get the direction of the node.
    // TODO: Search what could be in the media.class property.
    let node_direction = props
        .get("media.class")
        .and_then(|string| {
            let string = String::from(string);
            if string.contains("Input") || string.contains("Sink") {
                Some("Input")
            } else if string.contains("Output") || string.contains("Source") {
                Some("Output")
            } else {
                None
            }
        })
        .unwrap_or_default()
        .to_string();

    // Send the port to the front
    sender
        .send(MainOptions::CreateNode {
            id,
            permissions,
            props: node_props,
            name,
            node_direction,
            node_type,
        })
        .expect("ERROR: error at sending option to front");
}

// Create a port and send it to the front.
fn save_port(port: &GlobalObject<&DictRef>, sender: &mpsc::Sender<MainOptions>) {
    // println!("Port: {:?}", port);

    let id = port.id;
    let permissions = port.permissions;
    let props = port
        .props
        .as_ref()
        .expect("ERROR: error at getting port properties");

    // create a hashmap that will contain all the properties of the port
    let mut port_props = std::collections::HashMap::new();

    // iterate over the properties and add them to the vector
    for (key, value) in props.iter() {
        port_props.insert(key.to_string(), value.to_string());
    }

    // Get the node_id of the port.
    let node_id: u32 = props
        .get("node.id")
        .expect("ERROR: error at getting node id")
        .parse()
        .expect("ERROR: error at parsing node id");

    // Get the name of the port.
    let name = props.get("port.name").unwrap_or_default().to_string();

    // Get the direction of the port.
    let direction = props
        .get("port.direction")
        .and_then(|dir| {
            if dir.contains("in") {
                Some("Input")
            } else if dir.contains("out") {
                Some("Output")
            } else {
                None
            }
        })
        .unwrap_or_default()
        .to_string();

    // Send the port to the front.
    sender
        .send(MainOptions::CreatePort {
            id,
            permissions,
            props: port_props,
            node_id,
            name,
            direction,
        })
        .expect("ERROR: error at sending option to front");
}

// Create or modify link and send it to the front.
fn save_link(link: &GlobalObject<&DictRef>, sender: &mpsc::Sender<MainOptions>) {
    // println!("Link: {:?}", link);

    let sender = sender.clone();
    let all_data = ALL_DATA.lock().unwrap();

    let id = link.id;
    let permissions = link.permissions;
    let props = link
        .props
        .as_ref()
        .expect("ERROR: error at getting link properties");

    // create a hashmap that will contain all the properties of the link
    let mut link_props = std::collections::HashMap::new();

    // iterate over the properties and add them to the vector
    for (key, value) in props.iter() {
        link_props.insert(key.to_string(), value.to_string());
    }

    // From enum all_data, get only the PipewireData::Link
    let mut links = Vec::new();
    for data in all_data.iter() {
        if let PipewireData::Link(link) = data.1 {
            links.push(link.clone());
        }
    }
    drop(all_data);

    // Check if the link exists, if not create it.
    if !links.iter().any(|link| link.id == id) {
        // Get the output node id of the link.
        let output_node: u32 = props
            .get("link.output.node")
            .expect("ERROR: error at getting output node id")
            .parse()
            .expect("ERROR: error at parsing output node id");

        // Get the output port id of the link.
        let output_port: u32 = props
            .get("link.output.port")
            .expect("ERROR: error at getting output port id")
            .parse()
            .expect("ERROR: error at parsing output port id");

        // Get the input node id of the link.
        let input_node: u32 = props
            .get("link.input.node")
            .expect("ERROR: error at getting input node id")
            .parse()
            .expect("ERROR: error at parsing input node id");

        // Get the input port id of the link.
        let input_port: u32 = props
            .get("link.input.port")
            .expect("ERROR: error at getting input port id")
            .parse()
            .expect("ERROR: error at parsing input port id");

        // Send the link to the front.
        sender
            .send(MainOptions::CreateLink {
                id,
                permissions,
                props: link_props,
                output_node,
                output_port,
                input_node,
                input_port,
            })
            .expect("ERROR: error at sending option to front");
    }
}

// Create or modify client and send it to the front.
fn save_client(client: &GlobalObject<&DictRef>, sender: &mpsc::Sender<MainOptions>) {
    // println!("Client: {:?}", client);

    let sender = sender.clone();
    let all_data = ALL_DATA.lock().unwrap();

    let id = client.id;
    let permissions = client.permissions;
    let props = client
        .props
        .as_ref()
        .expect("ERROR: error at getting client properties");

    // create a hashmap that will contain all the properties of the client
    let mut client_props = std::collections::HashMap::new();

    // iterate over the properties and add them to the vector
    for (key, value) in props.iter() {
        client_props.insert(key.to_string(), value.to_string());
    }

    // From enum all_data, get only the PipewireData::Client
    let mut clients = Vec::new();
    for data in all_data.iter() {
        if let PipewireData::Client(client) = data.1 {
            clients.push(client.clone());
        }
    }
    drop(all_data);

    // Check if the client exists, if not create it.
    if !clients.iter().any(|client| client.id == id) {
        let pid: u32 = props
            .get("pipewire.sec.pid")
            .expect("ERROR: error at getting pid")
            .parse()
            .expect("ERROR: error at parsing pid");
        let application_name: String = props
            .get("application.name")
            .expect("ERROR: error at getting pid")
            .to_string();

        // Send the client to the front.
        sender
            .send(MainOptions::CreateClient {
                id,
                permissions,
                pid,
                application_name,
                props: client_props,
            })
            .expect("ERROR: error at sending option to front");
    }
}

// Link two ports.
fn link_ports(input_port_id: u32, output_port_id: u32, permanent: bool, core: &Core) -> Link {
    let mut ports = Vec::new();

    let all_data = ALL_DATA.lock().unwrap();
    // From enum all_data, get only the PipewireData::Port
    for data in all_data.iter() {
        if let PipewireData::Port(port) = data.1 {
            ports.push(port.clone());
        }
    }
    drop(all_data);

    // Get the input port.
    let input_port = ports
        .iter()
        .find(|port| port.id == input_port_id)
        .expect("ERROR: error at getting input port");

    // Get the output port.
    let output_port = ports
        .iter()
        .find(|port| port.id == output_port_id)
        .expect("ERROR: error at getting output port");

    // Create the link.
    return core
        .create_object::<Link>(
            // The actual name for a link factory might be different for your system,
            // you should probably obtain a factory from the registry.
            "link-factory",
            &properties! {
                "link.output.port" => output_port.id.to_string(),
                "link.input.port" => input_port.id.to_string(),
                "link.output.node" => output_port.node_id.to_string(),
                "link.input.node" => input_port.node_id.to_string(),
                "object.linger" => permanent.to_string(),
            },
        )
        .expect("ERROR: error at creating link");
}

// Unlink two ports.
fn unlink_ports(input_port_id: u32, output_port_id: u32, registry: &Registry) {
    let all_data = ALL_DATA.lock().unwrap();
    // From enum all_data, get only the PipewireData::Link
    let mut links = Vec::new();
    for data in all_data.iter() {
        if let PipewireData::Link(link) = data.1 {
            links.push(link.clone());
        }
    }

    // From enum all_data, get only the PipewireData::Port
    let mut ports = Vec::new();
    for data in all_data.iter() {
        if let PipewireData::Port(port) = data.1 {
            ports.push(port.clone());
        }
    }
    drop(all_data);

    // Get the output port.
    let output_port = ports
        .iter()
        .find(|port| port.id == output_port_id)
        .expect("ERROR: error at getting output port");

    // Get the input port.
    let input_port = ports
        .iter()
        .find(|port| port.id == input_port_id)
        .expect("ERROR: error at getting input port");

    // Get the link id.
    let link_id = links
        .iter()
        .find(|link| {
            link.output_node_id == output_port.node_id
                && link.output_port_id == output_port.id
                && link.input_node_id == input_port.node_id
                && link.input_port_id == input_port.id
        })
        .expect("ERROR: error at getting link id")
        .id;

    // Remove the link.
    registry.destroy_global(link_id);
}

fn link_nodes_name_to_id(
    nodes_name: String,
    input_node_id: u32,
    permanent: bool,
    core: &Core,
) -> Vec<Link> {
    let all_data = ALL_DATA.lock().unwrap();

    // From enum all_data, get only the PipewireData::Node
    let mut nodes = Vec::new();
    for data in all_data.iter() {
        if let PipewireData::Node(node) = data.1 {
            nodes.push(node.clone());
        }
    }
    drop(all_data);

    // get all nodes that has the name of "nodes_name"
    let output_nodes = nodes
        .iter()
        .filter(|node| node.name == nodes_name)
        .collect::<Vec<_>>();

    // get all output ports of the nodes.
    let mut output_ports = Vec::new();
    for node in output_nodes.iter() {
        for port in node.ports.iter() {
            if port.direction == "Output" {
                output_ports.push(port);
            }
        }
    }

    // split the output ports into two vectors, FR and FL
    let mut output_ports_fr = Vec::new();
    let mut output_ports_fl = Vec::new();
    for port in output_ports.iter() {
        if port.name.contains("FR") {
            output_ports_fr.push(port);
        } else if port.name.contains("FL") {
            output_ports_fl.push(port);
        }
    }

    // get the input node and its ports
    let input_node = nodes
        .iter()
        .find(|node| node.id == input_node_id)
        .expect("ERROR: error at finding input node");

    // get all input ports of the input node.
    let mut input_ports = Vec::new();
    for port in input_node.ports.iter() {
        if port.direction == "Input" {
            input_ports.push(port);
        }
    }

    // split the input ports into variables, FR and FL. That will contain the ports that will be linked.
    let input_port_fr = input_ports.iter().find(|port| port.name.contains("FR"));
    let input_port_fl = input_ports.iter().find(|port| port.name.contains("FL"));
    let input_port_mono = input_ports.iter().find(|port| port.name.contains("MONO"));

    let mut links = Vec::new();

    // if the input ports (fr and fl) are found, link every output port (fr and fl) to the input ports (fr and fl)
    if input_port_fr.is_some() && input_port_fl.is_some() {
        for port in output_ports_fr.iter() {
            let link = link_ports(input_port_fr.unwrap().id, port.id, permanent, core);
            links.push(link);
        }
        for port in output_ports_fl.iter() {
            let link = link_ports(input_port_fl.unwrap().id, port.id, permanent, core);
            links.push(link);
        }
    } else if input_port_mono.is_some() {
        // if the input ports (fr and fl) are not found, link every output port (fr and fl) to the default input port (mono)
        for port in output_ports_fr.iter() {
            let link = link_ports(input_port_mono.unwrap().id, port.id, permanent, core);
            links.push(link);
        }
        for port in output_ports_fl.iter() {
            let link = link_ports(input_port_mono.unwrap().id, port.id, permanent, core);
            links.push(link);
        }
    } else {
        // if the input ports (fr and fl) and the default input port (mono) are not found, print an error.
        if ENABLE_DEBUG.with(|f| *f.borrow()) {
            println!("ERROR: error at finding input ports, trying to link in every input port");
        }
        for port in output_ports_fr.iter() {
            for input_port in input_ports.iter() {
                let link = link_ports(input_port.id, port.id, permanent, core);
                links.push(link);
            }
        }
        for port in output_ports_fl.iter() {
            for input_port in input_ports.iter() {
                let link = link_ports(input_port.id, port.id, permanent, core);
                links.push(link);
            }
        }
    }

    return links;
}

fn unlink_nodes_name_to_id(nodes_name: String, input_node_id: u32, registry: &Registry) {
    let all_data = ALL_DATA.lock().unwrap();

    // From enum all_data, get only the PipewireData::Node
    let mut nodes = Vec::new();
    for data in all_data.iter() {
        if let PipewireData::Node(node) = data.1 {
            nodes.push(node.clone());
        }
    }
    drop(all_data);

    // get all nodes that has the name of "nodes_name"
    let output_nodes = nodes
        .iter()
        .filter(|node| node.name == nodes_name)
        .collect::<Vec<_>>();

    // get the input node and its ports
    let input_node = nodes
        .iter()
        .find(|node| node.id == input_node_id)
        .expect("ERROR: error at finding input node");

    // get all output ports of the nodes.
    let mut output_ports = Vec::new();
    for node in output_nodes.iter() {
        for port in node.ports.iter() {
            if port.direction == "Output" {
                output_ports.push(port);
            }
        }
    }

    // split the output ports into two vectors, FR and FL
    let mut output_ports_fr = Vec::new();
    let mut output_ports_fl = Vec::new();
    for port in output_ports.iter() {
        if port.name.contains("FR") {
            output_ports_fr.push(port);
        } else if port.name.contains("FL") {
            output_ports_fl.push(port);
        }
    }

    // get all input ports of the input node.
    let mut input_ports = Vec::new();
    for port in input_node.ports.iter() {
        if port.direction == "Input" {
            input_ports.push(port);
        }
    }

    // split the input ports into variables, FR and FL. That will contain the ports that will be linked.
    let input_port_fr = input_ports.iter().find(|port| port.name.contains("FR"));
    let input_port_fl = input_ports.iter().find(|port| port.name.contains("FL"));
    let input_port_mono = input_ports.iter().find(|port| port.name.contains("MONO"));

    // if the input ports (fr and fl) are found, unlink every output port (fr and fl) to the input ports (fr and fl)
    if input_port_fr.is_some() && input_port_fl.is_some() {
        for port in output_ports_fr.iter() {
            unlink_ports(port.id, input_port_fr.unwrap().id, registry);
        }
        for port in output_ports_fl.iter() {
            unlink_ports(port.id, input_port_fl.unwrap().id, registry);
        }
    } else if input_port_mono.is_some() {
        // if the input ports (fr and fl) are not found, unlink every output port (fr and fl) to the default input port (mono)
        for port in output_ports.iter() {
            unlink_ports(port.id, input_port_mono.unwrap().id, registry);
        }
    } else {
        // if the input ports (fr and fl) and the default input port (mono) are not found, print an error.
        if ENABLE_DEBUG.with(|f| *f.borrow()) {
            println!("ERROR: error at finding input ports, trying to unlink in every input port");
        }
        for port in output_ports.iter() {
            for input_port in input_ports.iter() {
                unlink_ports(port.id, input_port.id, registry);
            }
        }
    }
}

fn create_source(
    source_name: String,
    audio_position: String,
    channel_count: u32,
    permanent: bool,
    core: &Core,
) -> Node {
    return core
        .create_object::<Node>(
            &"adapter",
            &properties! {
                "media.class" => "Audio/Source/Virtual",
                "node.name" => "node-pipewire:".to_string() + &source_name,
                "node.nick" => source_name,
                "audio.position" => audio_position,
                "audio.channels" => channel_count.to_string(),
                "factory.name" => "support.null-audio-sink",
                "object.linger" => permanent.to_string(),
            },
        )
        .expect("error creating virtual source");
}

fn create_sink(
    sink_name: String,
    audio_position: String,
    channel_count: u32,
    permanent: bool,
    core: &Core,
) -> Node {
    return core
        .create_object::<Node>(
            &"adapter",
            &properties! {
                "media.class" => "Audio/Sink/Virtual",
                "node.name" => "node-pipewire:".to_string() + &sink_name,
                "node.nick" => sink_name,
                "audio.position" => audio_position,
                "audio.channels" => channel_count.to_string(),
                "factory.name" => "support.null-audio-sink",
                "object.linger" => permanent.to_string(),
            },
        )
        .expect("error creating virtual sink");
}

fn destroy_object(id: u32, registry: &Registry) {
    let all_data = ALL_DATA.lock().unwrap();

    // Get the object for this id
    let target = all_data
        .iter()
        .find(|obj| match obj.1 {
            PipewireData::Link(link) => link.id == id,
            PipewireData::Port(port) => port.id == id,
            PipewireData::Node(node) => node.id == id,
            PipewireData::Client(client) => client.id == id,
        })
        .expect("ERROR: error at finding target")
        .1
        .clone();
    drop(all_data);

    let allow;

    match target {
        PipewireData::Node(node) => {
            allow = node
                .props
                .get("node.name")
                .expect("ERROR: Node did not have a name prop")
                .starts_with("node-pipewire:");
        }
        PipewireData::Link(_) => allow = true,
        _ => {
            allow = false;
        }
    }

    if allow {
        if ENABLE_DEBUG.with(|f| *f.borrow()) {
            println!("Allowing to destroy object with id {}", id);
        }
        registry.destroy_global(id);
    } else if ENABLE_DEBUG.with(|f| *f.borrow()) {
        println!("Disallowing to destroy object with id {}", id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pipewire::core::PW_ID_CORE;
    use pipewire::registry::RegistryRc;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn do_roundtrip() {
        let mainloop = MainLoopRc::new(None).expect("ERROR: error at creating mainloop");
        let context = ContextRc::new(&mainloop, None).expect("ERROR: error at creating context");
        let core = context
            .connect_rc(None)
            .expect("ERROR: error at connecting context");

        let registry = core
            .get_registry_rc()
            .expect("ERROR: error at getting registry");

        roundtrip(&mainloop, &core, &registry);
    }

    fn roundtrip(mainloop: &MainLoopRc, core: &Core, registry: &RegistryRc) {
        // To comply with Rust's safety rules, we wrap this variable in an `Rc` and  a `Cell`.
        let done = Rc::new(Cell::new(false));

        // Create new reference for each variable so that they can be moved into the closure.
        let done_clone = done.clone();
        let loop_clone = mainloop.clone();

        // Trigger the sync event. The server's answer won't be processed until we start the main loop,
        // so we can safely do this before setting up a callback. This lets us avoid using a Cell.
        let pending = core.sync(0).expect("sync failed");

        let _listener_registry = registry
            .add_listener_local()
            .global({
                move |object| match object.type_ {
                    // To print out specific objects, match them here
                    // ObjectType::Client => println!("{:?}", object),
                    _ => {
                        // Ignore other types.
                    }
                }
            })
            .register();

        let _listener_core = core
            .add_listener_local()
            .done(move |id, seq| {
                if id == PW_ID_CORE && seq == pending {
                    done_clone.set(true);
                    loop_clone.quit();
                }
            })
            .register();

        while !done.get() {
            mainloop.run();
        }
    }
}
