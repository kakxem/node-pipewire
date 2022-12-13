mod pipewire_thread;

use lazy_static::lazy_static;
use neon::prelude::*;
use once_cell::sync::OnceCell;
use pipewire::registry::Permission;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc, Arc, Mutex}, time::Instant,
};
use tokio::runtime::Runtime;

#[derive(Clone, Debug)]
pub struct PipewirePort {
    pub id: u32,
    pub permissions: Permission,
    pub props: HashMap<String, String>,
    pub node_id: u32,
    pub name: String,
    pub direction: String,
}

impl PipewirePort {
    fn to_object<'a>(&self, cx: &mut FunctionContext<'a>) -> JsResult<'a, JsObject> {
        let obj = cx.empty_object();

        let js_id = cx.number(self.id);
        let js_permissions = cx.number(self.permissions.bits() as i32);
        let js_props = cx.string(serde_json::to_string(&self.props).unwrap());
        let js_node_id = cx.number(self.node_id as i32);
        let js_name = cx.string(self.name.clone());
        let js_direction = cx.string(self.direction.clone());

        obj.set(cx, "id", js_id)?;
        obj.set(cx, "permissions", js_permissions)?;
        obj.set(cx, "props", js_props)?;
        obj.set(cx, "node_id", js_node_id)?;
        obj.set(cx, "name", js_name)?;
        obj.set(cx, "direction", js_direction)?;

        Ok(obj)
    }
}
#[derive(Clone, Debug)]
pub struct PipewireNode {
    pub id: u32,
    pub permissions: Permission,
    pub props: HashMap<String, String>,
    pub name: String,
    pub node_direction: String,
    pub node_type: String,
    pub ports: Vec<PipewirePort>,
}

impl PipewireNode {
    fn to_object<'a>(&self, cx: &mut FunctionContext<'a>) -> JsResult<'a, JsObject> {
        let obj = cx.empty_object();

        let js_id = cx.number(self.id);
        let js_permissions = cx.number(self.permissions.bits() as i32);
        let js_props = cx.string(serde_json::to_string(&self.props).unwrap());
        let js_name = cx.string(self.name.clone());
        let js_node_direction = cx.string(self.node_direction.clone());
        let js_node_type = cx.string(self.node_type.clone());

        let js_ports = cx.empty_array();
        for (i, port) in self.ports.iter().enumerate() {
            let js_port = port.to_object(cx)?;
            js_ports.set(cx, i as u32, js_port)?;
        }

        obj.set(cx, "id", js_id)?;
        obj.set(cx, "permissions", js_permissions)?;
        obj.set(cx, "props", js_props)?;
        obj.set(cx, "name", js_name)?;
        obj.set(cx, "node_direction", js_node_direction)?;
        obj.set(cx, "node_type", js_node_type)?;
        obj.set(cx, "ports", js_ports)?;

        Ok(obj)
    }
}

#[derive(Clone, Debug)]
pub struct PipewireLink {
    pub id: u32,
    pub permissions: Permission,
    pub props: HashMap<String, String>,
    pub input_node_id: u32,
    pub input_port_id: u32,
    pub output_node_id: u32,
    pub output_port_id: u32,
}

impl PipewireLink {
    fn to_object<'a>(&self, cx: &mut FunctionContext<'a>) -> JsResult<'a, JsObject> {
        let obj = cx.empty_object();

        let js_id = cx.number(self.id);
        let js_permissions = cx.number(self.permissions.bits() as i32);
        let js_props = cx.string(serde_json::to_string(&self.props).unwrap());
        let js_input_node_id = cx.number(self.input_node_id as i32);
        let js_input_port_id = cx.number(self.input_port_id as i32);
        let js_output_node_id = cx.number(self.output_node_id as i32);
        let js_output_port_id = cx.number(self.output_port_id as i32);

        obj.set(cx, "id", js_id)?;
        obj.set(cx, "permissions", js_permissions)?;
        obj.set(cx, "props", js_props)?;
        obj.set(cx, "input_node_id", js_input_node_id)?;
        obj.set(cx, "input_port_id", js_input_port_id)?;
        obj.set(cx, "output_node_id", js_output_node_id)?;
        obj.set(cx, "output_port_id", js_output_port_id)?;

        Ok(obj)
    }
}

// create an enum that will contain all the data we need to store
#[derive(Clone, Debug)]
pub enum PipewireData {
    Link(PipewireLink),
    Port(PipewirePort),
    Node(PipewireNode),
}

// Create an enum with all the options that are available to send in front. (Pipewire thread -> Front)
enum MainOptions {
    // Create a node.
    CreateNode {
        id: u32,
        permissions: Permission,
        props: HashMap<String, String>,
        name: String,
        node_direction: String,
        node_type: String,
    },
    // Create a port.
    CreatePort {
        id: u32,
        permissions: Permission,
        props: HashMap<String, String>,
        node_id: u32,
        name: String,
        direction: String,
    },
    // Create a link.
    CreateLink {
        id: u32,
        permissions: Permission,
        props: HashMap<String, String>,
        input_node: u32,
        output_node: u32,
        input_port: u32,
        output_port: u32,
    },
    // Delete item (node, port, link).
    DeleteItem {
        id: u32,
    },
}

// Create an enum with all the options that are available to send in back. (Front -> Pipewire thread)
enum PipewireOptions {
    // Close the mainloop of the pipewire thread.
    CloseThread,
    // TODO: add more options (LinkNodes, UnLinkNodes, Delete...).
    LinkPorts {
        input_port: u32,
        output_port: u32,
    },
    UnLinkPorts {
        input_port: u32,
        output_port: u32,
    },
    LinkNodesNameToId {
        output_nodes_name: String,
        input_node_id: u32,
    },
    UnLinkNodesNameToId {
        output_nodes_name: String,
        input_node_id: u32,
    },
}

// create a global variable with RefCell to store all the data we need
lazy_static! {
    static ref ALL_DATA: Arc<Mutex<HashMap<u32, PipewireData>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// create a global variable that will store the sender of the main thread
thread_local! {
    static PW_SENDER: RefCell<Option<pipewire::channel::Sender<PipewireOptions>>> = RefCell::new(None);
    static ENABLE_DEBUG: RefCell<bool> = RefCell::new(false);
}

fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

fn create_pw_thread(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Mini-Schema:
    // - We can send option to pipewire with pw_sender and we receive options from pipewire with pw_receiver.

    // get the debug boolean from the arguments
    let debug_argument = cx
        .argument_opt(0)
        .map(|v| Ok(v.downcast_or_throw::<JsBoolean, _>(&mut cx)?.value(&mut cx)))
        .transpose()?;

    if let Some(debug) = debug_argument {
        ENABLE_DEBUG.with(|v| *v.borrow_mut() = debug);
    }

    let enable_debug = ENABLE_DEBUG.with(|v| *v.borrow());

    // start a sender and receiver to communicate with the pipewire thread
    let (main_sender, main_receiver) = mpsc::channel();
    // start a sender and receiver to communicate with the main thread
    let (pw_sender, pw_receiver) = pipewire::channel::channel();

    // store the sender in a global variable
    PW_SENDER.with(|sender| {
        *sender.borrow_mut() = Some(pw_sender);
    });

    // Start the pipewire thread.
    // TODO: make the thread global to be able to stop it.
    let _pw_thread = std::thread::spawn(move || {
        pipewire_thread::pw_thread(main_sender, pw_receiver, enable_debug);
    });

    let mut num_changes = 0;

    // Listen the main_receiver the options from "MainOptions" struct
    let _receiver = std::thread::spawn(move || loop {
        let option = main_receiver.recv().unwrap();
        match option {
            MainOptions::CreateNode {
                id,
                permissions,
                props,
                name,
                node_direction,
                node_type,
            } => {
                if enable_debug {
                    println!(
                        "{} + Node added: id: {}, name: {}, direction: {:?},",
                        num_changes, id, name, node_direction
                    );
                }
                num_changes += 1;

                // add port to ALL_DATA
                let mut all_data = ALL_DATA.lock().unwrap();

                all_data.insert(
                    id,
                    PipewireData::Node(PipewireNode {
                        id,
                        permissions,
                        props,
                        name,
                        node_direction,
                        node_type,
                        ports: Vec::new(),
                    }),
                );
            }
            MainOptions::CreatePort {
                id,
                permissions,
                props,
                node_id,
                name,
                direction,
            } => {
                if enable_debug {
                    println!(
                        "{} + Port added: id: {}, node_id: {}, name: {}, direction: {:?}",
                        num_changes, id, node_id, name, direction
                    );
                }
                num_changes += 1;

                // add port to ALL_DATA
                let mut all_data = ALL_DATA.lock().unwrap();

                // save the new port in a var
                let new_port = PipewirePort {
                    id,
                    permissions,
                    props,
                    node_id,
                    name,
                    direction,
                };

                all_data.insert(id, PipewireData::Port(new_port.clone()));

                // search the node with the id node_id and add the port to it
                for (_, value) in all_data.iter_mut() {
                    match value {
                        PipewireData::Node(node) => {
                            if node.id == node_id {
                                node.ports.push(new_port.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
            MainOptions::CreateLink {
                id,
                permissions,
                props,
                input_node,
                input_port,
                output_node,
                output_port,
            } => {
                if enable_debug {
                    println!(
                            "{} + Link added: id: {}, node_from: {}, port_from: {}, node_to: {}, port_to: {}",
                            num_changes, id, input_node, input_port, output_node, output_port
                        );
                }
                num_changes += 1;

                // add link to ALL_DATA
                let mut all_data = ALL_DATA.lock().unwrap();

                all_data.insert(
                    id,
                    PipewireData::Link(PipewireLink {
                        id,
                        permissions,
                        props,
                        input_node_id: input_node,
                        input_port_id: input_port,
                        output_node_id: output_node,
                        output_port_id: output_port,
                    }),
                );
            }
            MainOptions::DeleteItem { id } => {
                // remove item from ALL_DATA
                let mut all_data = ALL_DATA.lock().unwrap();

                // the next if is only to debug purposes
                if enable_debug {
                    if let Some(item) = all_data.get(&id) {
                        match item {
                            PipewireData::Node(node) => {
                                println!(
                                    "{} - Removing node: id: {}, name: {}",
                                    num_changes, node.id, node.name
                                );
                            }
                            PipewireData::Port(port) => {
                                println!(
                                    "{} - Removing port: id: {}, name: {}",
                                    num_changes, port.id, port.name
                                );
                            }
                            PipewireData::Link(link) => {
                                println!("{} - Removing link: id: {}, node_from: {}, port_from: {}, node_to: {}, port_to: {}", num_changes, link.id, link.input_node_id, link.input_port_id, link.output_node_id, link.output_port_id);
                            }
                        }
                    } else {
                        println!("{} - Removing unknown: {}", num_changes, id);
                    }
                }

                if let Some(_) = all_data.remove(&id) {
                    num_changes += 1;
                } else {
                    if enable_debug {
                        println!("{} - Error removing item: {}", num_changes, id);
                    }
                }
            }
        }
    });

    // save the enable_debug in a global variable
    ENABLE_DEBUG.with(|debug| {
        *debug.borrow_mut() = enable_debug;
    });

    Ok(cx.undefined())
}

fn close_pw_thread(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // get the pw_sender from the context data
    let temp_pw_sender: pipewire::channel::Sender<PipewireOptions> = PW_SENDER.with(|pw_sender| {
        pw_sender
            .borrow_mut()
            .take()
            .expect("pw_sender not set in context data")
    });

    // send the message to the pw thread
    let result = temp_pw_sender.send(PipewireOptions::CloseThread);

    if let Err(_) = result {
        if ENABLE_DEBUG.with(|debug| *debug.borrow()) {
            println!("Error sending message to pw thread");
        }
    }

    Ok(cx.undefined())
}

fn get_links(mut cx: FunctionContext) -> JsResult<JsArray> {
    let output = JsArray::new(&mut cx, 0);

    let all_data = ALL_DATA.lock().unwrap();

    let mut counter = 0;
    // From all_data, get all links and add them to the output array
    for (_, data) in all_data.iter() {
        match data {
            PipewireData::Link(link) => {
                let js_link = link.to_object(&mut cx);

                // if js_link result is Ok, add it to the output array
                if let Ok(js_link) = js_link {
                    output.set(&mut cx, counter, js_link).unwrap();
                    counter += 1;
                }
            }
            _ => {}
        }
    }
    Ok(output)
}

fn get_ports(mut cx: FunctionContext) -> JsResult<JsArray> {
    let output = JsArray::new(&mut cx, 0);

    let all_data = ALL_DATA.lock().unwrap();

    let mut counter = 0;
    // From all_data, get all ports and add them to the output array
    for (_, data) in all_data.iter() {
        match data {
            PipewireData::Port(port) => {
                let js_port = port.to_object(&mut cx);

                // if js_port result is Ok, add it to the output array
                if let Ok(js_port) = js_port {
                    output.set(&mut cx, counter, js_port).unwrap();
                    counter += 1;
                }
            }
            _ => {}
        }
    }

    Ok(output)
}

fn get_nodes(mut cx: FunctionContext) -> JsResult<JsArray> {
    let output = JsArray::new(&mut cx, 0);

    let all_data = ALL_DATA.lock().unwrap();

    let mut counter = 0;
    // From all_data, get all nodes and add them to the output array
    for (_, data) in all_data.iter() {
        match data {
            PipewireData::Node(node) => {
                let js_node = node.to_object(&mut cx);

                // if js_node result is Ok, add it to the output array
                if let Ok(js_node) = js_node {
                    output.set(&mut cx, counter, js_node).unwrap();
                    counter += 1;
                }
            }
            _ => {}
        }
    }
    Ok(output)
}

fn get_output_nodes(mut cx: FunctionContext) -> JsResult<JsArray> {
    let output = JsArray::new(&mut cx, 0);

    let all_data = ALL_DATA.lock().unwrap();

    let mut counter = 0;
    // From all_data, get all output nodes and add them to the output array
    for (_, data) in all_data.iter() {
        match data {
            PipewireData::Node(node) => {
                if node.node_direction == "Output" {
                    let js_node = node.to_object(&mut cx);

                    // if js_node result is Ok, add it to the output array
                    if let Ok(js_node) = js_node {
                        output.set(&mut cx, counter, js_node).unwrap();
                        counter += 1;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(output)
}

fn get_input_nodes(mut cx: FunctionContext) -> JsResult<JsArray> {
    let output = JsArray::new(&mut cx, 0);

    let all_data = ALL_DATA.lock().unwrap();

    let mut counter = 0;
    // From all_data, get all input nodes and add them to the output array
    for (_, data) in all_data.iter() {
        match data {
            PipewireData::Node(node) => {
                if node.node_direction == "Input" {
                    let js_node = node.to_object(&mut cx);

                    // if js_node result is Ok, add it to the output array
                    if let Ok(js_node) = js_node {
                        output.set(&mut cx, counter, js_node).unwrap();
                        counter += 1;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(output)
}

fn link_nodes_name_to_id(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let input_nodes_name = cx.argument::<JsString>(0)?;
    let output_node_id = cx.argument::<JsNumber>(1)?;

    let input_nodes_name = input_nodes_name.value(&mut cx);
    let output_node_id = output_node_id.value(&mut cx) as u32;

    // get the pw_sender from the context data
    let temp_pw_sender: pipewire::channel::Sender<PipewireOptions> = PW_SENDER.with(|pw_sender| {
        pw_sender
            .borrow_mut()
            .take()
            .expect("pw_sender not set in context data")
    });

    // send the message to the pw thread
    let result = temp_pw_sender.send(PipewireOptions::LinkNodesNameToId {
        output_nodes_name: input_nodes_name,
        input_node_id: output_node_id,
    });

    // put the pw_sender back in the context data
    PW_SENDER.with(|pw_sender| {
        pw_sender.borrow_mut().replace(temp_pw_sender);
    });

    if let Err(_) = result {
        if ENABLE_DEBUG.with(|debug| *debug.borrow()) {
            println!("Error sending message to pw thread");
        }
    }

    Ok(cx.undefined())
}

fn unlink_nodes_name_to_id(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let output_nodes_name = cx.argument::<JsString>(0)?;
    let input_node_id = cx.argument::<JsNumber>(1)?;

    let output_nodes_name = output_nodes_name.value(&mut cx);
    let input_node_id = input_node_id.value(&mut cx) as u32;

    // get the pw_sender from the context data
    let temp_pw_sender: pipewire::channel::Sender<PipewireOptions> = PW_SENDER.with(|pw_sender| {
        pw_sender
            .borrow_mut()
            .take()
            .expect("pw_sender not set in context data")
    });

    // send the message to the pw thread
    let result = temp_pw_sender.send(PipewireOptions::UnLinkNodesNameToId {
        output_nodes_name,
        input_node_id,
    });

    // put the pw_sender back in the context data
    PW_SENDER.with(|pw_sender| {
        pw_sender.borrow_mut().replace(temp_pw_sender);
    });

    if let Err(_) = result {
        if ENABLE_DEBUG.with(|debug| *debug.borrow()) {
            println!("Error sending message to pw thread");
        }
    }

    Ok(cx.undefined())
}

fn link_ports(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let input_port_id = cx.argument::<JsNumber>(0)?;
    let output_port_id = cx.argument::<JsNumber>(1)?;

    let input_port_id = input_port_id.value(&mut cx) as u32;
    let output_port_id = output_port_id.value(&mut cx) as u32;

    // get the pw_sender from the context data
    let temp_pw_sender: pipewire::channel::Sender<PipewireOptions> = PW_SENDER.with(|pw_sender| {
        pw_sender
            .borrow_mut()
            .take()
            .expect("pw_sender not set in context data")
    });

    // send the message to the pw thread
    let result = temp_pw_sender.send(PipewireOptions::LinkPorts {
        input_port: input_port_id,
        output_port: output_port_id,
    });

    // put the pw_sender back in the context data
    PW_SENDER.with(|pw_sender| {
        pw_sender.borrow_mut().replace(temp_pw_sender);
    });

    if let Err(_) = result {
        if ENABLE_DEBUG.with(|debug| *debug.borrow()) {
            println!("Error sending message to pw thread");
        }
    }

    Ok(cx.undefined())
}

fn unlink_ports(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let input_port_id = cx.argument::<JsNumber>(0)?;
    let output_port_id = cx.argument::<JsNumber>(1)?;

    let input_port_id = input_port_id.value(&mut cx) as u32;
    let output_port_id = output_port_id.value(&mut cx) as u32;

    // get the pw_sender from the context data
    let temp_pw_sender: pipewire::channel::Sender<PipewireOptions> = PW_SENDER.with(|pw_sender| {
        pw_sender
            .borrow_mut()
            .take()
            .expect("pw_sender not set in context data")
    });

    // send the message to the pw thread
    let result = temp_pw_sender.send(PipewireOptions::UnLinkPorts {
        input_port: input_port_id,
        output_port: output_port_id,
    });

    // put the pw_sender back in the context data
    PW_SENDER.with(|pw_sender| {
        pw_sender.borrow_mut().replace(temp_pw_sender);
    });

    if let Err(_) = result {
        if ENABLE_DEBUG.with(|debug| *debug.borrow()) {
            println!("Error sending message to pw thread");
        }
    }

    Ok(cx.undefined())
}

fn get_current_nodes(
    node_name: String,
    node_directions: Vec<String>,
) -> Result<Vec<PipewireNode>, String> {
    let all_data = ALL_DATA.lock().unwrap();
    let mut new_nodes = Vec::new();
    for data in all_data.iter() {
        if let PipewireData::Node(node) = data.1 {
            if node.name.contains(node_name.as_str())
                && node_directions.contains(&node.node_direction)
            {
                new_nodes.push(node.clone());
            }
        }
    }
    drop(all_data);

    Ok(new_nodes)
}

fn wait_for_new_node(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let rt = runtime(&mut cx)?;
    let node_name = cx.argument::<JsString>(0)?;
    let node_direction = cx.argument::<JsString>(1)?;
    let timeout = cx.argument::<JsNumber>(2)?;
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    let node_name = node_name.value(&mut cx);
    let node_direction = node_direction.value(&mut cx);
    let timeout = timeout.value(&mut cx) as u128;

    let mut node_directions = Vec::new();
    if node_direction == "Input" || node_direction == "Output" {
        node_directions.push(node_direction);
    } else {
        node_directions.push("Input".to_string());
        node_directions.push("Output".to_string());
    }
    
    // get the actual nodes from the context data
    let nodes = get_current_nodes(node_name.clone(), node_directions.clone());

    match nodes {
        Ok(nodes) => {
            // get the actual time
            let start_time = Instant::now();
            // start async task
            rt.spawn(async move {
                let mut _node = None;
        
                loop {
                    // get the actual nodes from the context data
                    let new_nodes = get_current_nodes(node_name.clone(), node_directions.clone()).unwrap();
        
                    // check if the length of the nodes is different
                    if new_nodes.len() > 0 {
                        ENABLE_DEBUG.with(|debug| {
                            if *debug.borrow() {
                                println!("Old nodes:");
                                for node in nodes.iter() {
                                    println!("{} - {}", node.name, node.id);
                                }
                                println!("");
                                println!("New nodes:");
                                for node in new_nodes.iter() {
                                    println!("{} - {}", node.name, node.id);
                                }
                            }
                        });
        
                        // get the new node compared to the old nodes
                        for new_node in new_nodes.iter() {
                            let mut found = false;
                            for node in nodes.iter() {
                                if new_node.id == node.id {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                _node = Some(new_node.clone());
                                break;
                            }
                        }
        
                        if _node.is_some() {
                            break;
                        }
                    }
        
                    // check if the timeout is reached (default 5 seconds)
                    if start_time.elapsed().as_millis() > timeout {
                        break;
                    }

                    // wait 500ms (Maybe this can be changed to a lower value)
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
        
                // defer the result
                deferred.settle_with(&channel, move |mut cx_task| match _node {
                    Some(node) => {
                        let obj = cx_task.empty_object();
                        let id = cx_task.number(node.id);
                        let permissions = cx_task.number(node.permissions.bits() as i32);
                        let props = cx_task.string(serde_json::to_string(&node.props).unwrap());
                        let name = cx_task.string(node.name);
                        let node_direction = cx_task.string(node.node_direction);
                        let node_type = cx_task.string(node.node_type);
                        let ports = cx_task.empty_array();
        
                        for (i, port) in node.ports.iter().enumerate() {
                            let port_obj = cx_task.empty_object();
        
                            let port_id = cx_task.number(port.id);
                            let port_permissions = cx_task.number(port.permissions.bits() as i32);
                            let port_props = cx_task.string(serde_json::to_string(&port.props).unwrap());
                            let port_node_id = cx_task.number(port.node_id);
                            let port_name = cx_task.string(port.name.clone());
                            let port_direction = cx_task.string(port.direction.clone());
        
                            port_obj.set(&mut cx_task, "id", port_id).unwrap();
                            port_obj
                                .set(&mut cx_task, "permissions", port_permissions)
                                .unwrap();
                            port_obj.set(&mut cx_task, "props", port_props).unwrap();
                            port_obj.set(&mut cx_task, "node_id", port_node_id).unwrap();
                            port_obj.set(&mut cx_task, "name", port_name).unwrap();
                            port_obj
                                .set(&mut cx_task, "direction", port_direction)
                                .unwrap();
        
                            ports.set(&mut cx_task, i as u32, port_obj).unwrap();
                        }
        
                        obj.set(&mut cx_task, "id", id).unwrap();
                        obj.set(&mut cx_task, "permissions", permissions).unwrap();
                        obj.set(&mut cx_task, "props", props).unwrap();
                        obj.set(&mut cx_task, "name", name).unwrap();
                        obj.set(&mut cx_task, "node_direction", node_direction)
                            .unwrap();
                        obj.set(&mut cx_task, "node_type", node_type).unwrap();
                        obj.set(&mut cx_task, "ports", ports).unwrap();
        
                        Ok(obj)
                    }
                    None => cx_task.throw_error("No node found"),
                });
            });
        
        },
        Err(_) => {
            return cx.throw_error("Error getting the nodes"); 
        } 
    };

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createPwThread", create_pw_thread)?;
    cx.export_function("closePwThread", close_pw_thread)?;
    cx.export_function("getLinks", get_links)?;
    cx.export_function("getPorts", get_ports)?;
    cx.export_function("getNodes", get_nodes)?;
    cx.export_function("getOutputNodes", get_output_nodes)?;
    cx.export_function("getInputNodes", get_input_nodes)?;
    cx.export_function("linkNodesNameToId", link_nodes_name_to_id)?;
    cx.export_function("unlinkNodesNameToId", unlink_nodes_name_to_id)?;
    cx.export_function("linkPorts", link_ports)?;
    cx.export_function("unlinkPorts", unlink_ports)?;
    cx.export_function("waitForNewNode", wait_for_new_node)?;
    Ok(())
}
