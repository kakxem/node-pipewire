const library = require("./index.node");

const createPwThread = () => {
  return library.createPwThread();
};

const closePwThread = () => {
  return library.closePwThread();
};

const getLinks = () => {
  return library.getLinks().filter((link) => link !== "");
};

const getPorts = () => {
  return library.getPorts().filter((port) => port !== "");
};

const getNodes = () => {
  return library.getNodes().filter((node) => node !== "");
};

const getOutputNodes = () => {
  return library.getOutputNodes().filter((node) => node !== "");
};

const getInputNodes = () => {
  return library.getInputNodes().filter((node) => node !== "");
};

const linkNodesNameToId = (nodeName, node_id) => {
  return library.linkNodesNameToId(nodeName, node_id);
};

const unlinkNodesNameToId = (nodeName, node_id) => {
  return library.unlinkNodesNameToId(nodeName, node_id);
};

const linkPorts = (input_port_id, output_port_id) => {
  return library.linkPorts(input_port_id, output_port_id);
};

const unlinkPorts = (input_port_id, output_port_id) => {
  return library.unlinkPorts(input_port_id, output_port_id);
};

const getPortsFromNode = (node) => {
  const temp = node.ports;
  const ports = [];
  for (let i = 0; i < temp.length; i++) {
    ports.push(JSON.parse(temp[i]));
  }
  return ports;
};

const getInputNodesName = () => {
  const temp = library.getInputNodes().filter((node) => node !== "");
  const nodes = [];
  for (let i = 0; i < temp.length; i++) {
    nodes.push(temp[i].name);
  }
  return nodes;
};

const getOutputNodesName = () => {
  const temp = library.getOutputNodes().filter((node) => node !== "");
  const nodes = [];
  for (let i = 0; i < temp.length; i++) {
    nodes.push(temp[i].name);
  }
  return nodes;
};

module.exports = {
  createPwThread,
  closePwThread,
  getPorts,
  getLinks,
  getNodes,
  getOutputNodes,
  getInputNodes,
  linkNodesNameToId,
  unlinkNodesNameToId,
  linkPorts,
  unlinkPorts,
  getPortsFromNode,
  getInputNodesName,
  getOutputNodesName,
};
