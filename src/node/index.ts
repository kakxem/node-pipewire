import type { PipewireLink, PipewirePort, PipewireNode } from "./types";
const library = require("./../../build/index.node");

export function createPwThread() {
  return library.createPwThread();
}

export function closePwThread() {
  return library.closePwThread();
}

export function getLinks(): PipewireLink[] {
  let temp = library.getLinks();
  let links: PipewireLink[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i].id) {
      links.push(temp[i]);
    }
  }
  return links;
}

export function getPorts(): PipewirePort[] {
  let temp = library.getPorts();
  let ports: PipewirePort[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i].id) {
      ports.push(temp[i]);
    }
  }
  return ports;
}

export function getNodes(): PipewireNode[] {
  let temp = library.getNodes();
  let nodes: PipewireNode[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i].id) {
      nodes.push(temp[i]);
    }
  }
  return nodes;
}

export function getOutputNodes(): PipewireNode[] {
  let temp = library.getOutputNodes();
  let nodes: PipewireNode[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i].id) {
      nodes.push(temp[i]);
    }
  }
  return nodes;
}

export function getInputNodes(): PipewireNode[] {
  let temp = library.getOutputNodes();
  let nodes: PipewireNode[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i].id) {
      nodes.push(temp[i]);
    }
  }
  return nodes;
}

export function linkNodesNameToId(nodeName: string, nodeId: number) {
  return library.linkNodesNameToId(nodeName, nodeId);
}

export function unlinkNodesNameToId(nodeName: string, nodeId: number) {
  return library.unlinkNodesNameToId(nodeName, nodeId);
}

export function linkPorts(inputPortId: number, outputPortId: number) {
  return library.linkPorts(inputPortId, outputPortId);
}

export function unlinkPorts(inputPortId: number, outputPortId: number) {
  return library.unlinkPorts(inputPortId, outputPortId);
}

export function getInputNodesName() {
  const temp = getInputNodes();
  const nodes = [];
  for (let i = 0; i < temp.length; i++) {
    nodes.push(temp[i].name);
  }
  return nodes;
}

export function getOutputNodesName() {
  const temp = getOutputNodes();
  const nodes = [];
  for (let i = 0; i < temp.length; i++) {
    nodes.push(temp[i].name);
  }
  return nodes;
}
