import type { PipewireLink, PipewirePort, PipewireNode } from "./types";

// eslint-disable-next-line @typescript-eslint/no-var-requires
const library = require("./../../build/index.node");

export function createPwThread() {
  return library.createPwThread();
}

export function closePwThread() {
  return library.closePwThread();
}

export function getLinks(): PipewireLink[] {
  const temp: PipewireLink[] = library.getLinks();
  const links: PipewireLink[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i].id) {
      links.push(temp[i]);
    }
  }
  return links;
}

export function getPorts(): PipewirePort[] {
  const temp: PipewirePort[] = library.getPorts();
  const ports: PipewirePort[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i]?.id) {
      ports.push(temp[i]);
    }
  }
  return ports;
}

export function getNodes(): PipewireNode[] {
  const temp: PipewireNode[] = library.getNodes();
  const nodes: PipewireNode[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i]?.id) {
      nodes.push(temp[i]);
    }
  }
  return nodes;
}

export function getOutputNodes(): PipewireNode[] {
  const temp: PipewireNode[] = library.getOutputNodes();
  const nodes: PipewireNode[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i]?.id) {
      nodes.push(temp[i]);
    }
  }
  return nodes;
}

export function getInputNodes(): PipewireNode[] {
  const temp = library.getOutputNodes();
  const nodes: PipewireNode[] = [];
  for (let i = 0; i < temp.length; i++) {
    if (temp[i]?.id) {
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

export function getInputNodesName(): string[] {
  const temp = getInputNodes();
  const nodes = [];
  for (let i = 0; i < temp.length; i++) {
    nodes.push(temp[i]?.name);
  }
  return nodes;
}

export function getOutputNodesName(): string[] {
  const temp = getOutputNodes();
  const nodes = [];
  for (let i = 0; i < temp.length; i++) {
    nodes.push(temp[i]?.name);
  }
  return nodes;
}
