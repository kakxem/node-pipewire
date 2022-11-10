mod pipewire_thread;

use lazy_static::lazy_static;
use neon::prelude::*;
use pipewire::registry::Permission;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};

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
}

fn create_pw_thread(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Mini-Schema:
    // - We can send option to pipewire with pw_sender and we receive options from pipewire with pw_receiver.

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
        pipewire_thread::pw_thread(main_sender, pw_receiver);
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
                println!(
                    "{} + Node added: id: {}, name: {}, direction: {:?},",
                    num_changes, id, name, node_direction
                );
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
                println!(
                    "{} + Port added: id: {}, node_id: {}, name: {}, direction: {:?}",
                    num_changes, id, node_id, name, direction
                );
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
                println!(
                        "{} + Link added: id: {}, node_from: {}, port_from: {}, node_to: {}, port_to: {}",
                        num_changes, id, input_node, input_port, output_node, output_port
                    );
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

                if let Some(_) = all_data.remove(&id) {
                    num_changes += 1;
                } else {
                    println!("{} - Error removing item: {}", num_changes, id);
                }
            }
        }
    });

    //

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
        println!("Error sending message to pw thread");
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
        println!("Error sending message to pw thread");
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
        println!("Error sending message to pw thread");
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
        println!("Error sending message to pw thread");
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
        println!("Error sending message to pw thread");
    }

    Ok(cx.undefined())
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
    Ok(())
}
