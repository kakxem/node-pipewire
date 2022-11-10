use crate::{MainOptions, PipewireData, PipewireOptions, ALL_DATA};
use pipewire::{
    link::Link,
    prelude::*,
    properties,
    registry::{GlobalObject, Registry},
    spa::ForeignDict,
    types::ObjectType,
    Context, Core, MainLoop,
};

use std::sync::mpsc;

pub(super) fn pw_thread(
    front_sender: mpsc::Sender<MainOptions>,
    pw_receiver: pipewire::channel::Receiver<PipewireOptions>,
) {
    // Basic setup of pipewire thread
    let mainloop = MainLoop::new().expect("ERROR: error at creating mainloop");
    let context = Context::new(&mainloop).expect("ERROR: error at creating context");
    let core = context
        .connect(None)
        .expect("ERROR: error at connecting context");

    let registry = core
        .get_registry()
        .expect("ERROR: error at getting registry");

    // Listen the pw_receiver the options from "PipewireOptions" struct
    let _receiver = pw_receiver.attach(&mainloop, {
        let mainloop = mainloop.clone();
        let core = core.clone();
        let registry = core
            .get_registry()
            .expect("ERROR: error at getting registry");
        move |msg| match msg {
            PipewireOptions::CloseThread => {
                println!("Closing pipewire thread");
                mainloop.quit();
            }
            PipewireOptions::LinkNodesNameToId {
                output_nodes_name,
                input_node_id,
            } => {
                println!(
                    "Linking nodes: {:?} -> {:?}",
                    output_nodes_name, input_node_id
                );
                link_nodes_name_to_id(output_nodes_name, input_node_id, &core)
            }
            PipewireOptions::LinkPorts {
                input_port,
                output_port,
            } => {
                println!("Linking ports: {:?} -> {:?}", input_port, output_port);
                link_ports(input_port, output_port, &core);
            }
            PipewireOptions::UnLinkNodesNameToId {
                output_nodes_name,
                input_node_id,
            } => {
                println!(
                    "Unlinking nodes: {:?} -> {:?}",
                    output_nodes_name, input_node_id
                );
                unlink_nodes_name_to_id(output_nodes_name, input_node_id, &registry)
            }
            PipewireOptions::UnLinkPorts {
                input_port,
                output_port,
            } => {
                println!("Unlinking ports: {:?} -> {:?}", input_port, output_port);
                unlink_ports(input_port, output_port, &registry);
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
                _ => {
                    // Ignore other types.
                }
            }
        })
        .global_remove({
            let sender = front_sender.clone();

            move |id| {
                sender
                    .send(MainOptions::DeleteItem { id })
                    .expect("ERROR: error at sending delete to front");
            }
        })
        .register();

    mainloop.run();
}

// Create a node and send it to the front.
fn save_node(node: &GlobalObject<ForeignDict>, sender: &mpsc::Sender<MainOptions>) {
    // println!("Node: {:?}", node);

    let id = node.id;
    let permissions = node.permissions;
    let props = node
        .props
        .as_ref()
        .expect("ERROR: error at getting node properties")
        .clone();

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
fn save_port(port: &GlobalObject<ForeignDict>, sender: &mpsc::Sender<MainOptions>) {
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
fn save_link(link: &GlobalObject<ForeignDict>, sender: &mpsc::Sender<MainOptions>) {
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

// Link two ports.
fn link_ports(input_port_id: u32, output_port_id: u32, core: &Core) {
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
    core.create_object::<Link, _>(
        // The actual name for a link factory might be different for your system,
        // you should probably obtain a factory from the registry.
        "link-factory",
        &properties! {
            "link.output.port" => output_port.id.to_string(),
            "link.input.port" => input_port.id.to_string(),
            "link.output.node" => output_port.node_id.to_string(),
            "link.input.node" => input_port.node_id.to_string(),
            "object.linger" => "1"
        },
    )
    .expect("ERROR: error at creating link");
}

// Unlink two ports.
fn unlink_ports(output_port_id: u32, input_port_id: u32, registry: &Registry) {
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

fn link_nodes_name_to_id(nodes_name: String, input_node_id: u32, core: &Core) {
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
    let default_input_port = input_ports.iter().find(|port| port.name == "MONO");

    // if the input ports (fr and fl) are found, link every output port (fr and fl) to the input ports (fr and fl)
    if input_port_fr.is_some() && input_port_fl.is_some() {
        for port in output_ports_fr.iter() {
            link_ports(input_port_fr.unwrap().id, port.id, core);
        }
        for port in output_ports_fl.iter() {
            link_ports(input_port_fl.unwrap().id, port.id, core);
        }
    } else if default_input_port.is_some() {
        // if the input ports (fr and fl) are not found, link every output port (fr and fl) to the default input port (mono)
        for port in output_ports_fr.iter() {
            link_ports(default_input_port.unwrap().id, port.id, core);
        }
        for port in output_ports_fl.iter() {
            link_ports(default_input_port.unwrap().id, port.id, core);
        }
    } else {
        // if the input ports (fr and fl) and the default input port (mono) are not found, print an error.
        println!("ERROR: error at finding input ports");
    }
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
    let default_input_port = input_ports.iter().find(|port| port.name == "MONO");

    // if the input ports (fr and fl) are found, unlink every output port (fr and fl) to the input ports (fr and fl)
    if input_port_fr.is_some() && input_port_fl.is_some() {
        for port in output_ports_fr.iter() {
            unlink_ports(port.id, input_port_fr.unwrap().id, registry);
        }
        for port in output_ports_fl.iter() {
            unlink_ports(port.id, input_port_fl.unwrap().id, registry);
        }
    } else if default_input_port.is_some() {
        // if the input ports (fr and fl) are not found, unlink every output port (fr and fl) to the default input port (mono)
        for port in output_ports.iter() {
            unlink_ports(port.id, default_input_port.unwrap().id, registry);
        }
    } else {
        // if the input ports (fr and fl) and the default input port (mono) are not found, print an error.
        println!("ERROR: error at finding input ports");
    }
}
