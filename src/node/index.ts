// Typescript definitions for node-pipewire
interface PipewirePort {
  id: number;
  permissions: number;
  props: string;
  node_id: number;
  name: string;
  direction: string;
}

interface PipewireNode {
  id: number;
  permissions: number;
  props: string;
  name: string;
  node_direction: string;
  node_type: string;
  ports: PipewirePort[];
}

interface PipewireLink {
  id: number;
  permissions: number;
  props: string;
  input_node_id: number;
  input_port_id: number;
  output_node_id: number;
  output_port_id: number;
}

type NodeDirection = "Input" | "Output" | "Both";

// eslint-disable-next-line @typescript-eslint/no-var-requires
const library = require("./index.node");

export function createPwThread(enableDebug?: boolean) {
  library.createPwThread(enableDebug ?? false);
}

// This feature is not yet implemented in the library
/* export function closePwThread() {
  return library.closePwThread();
} */

export function getLinks(): PipewireLink[] {
  const temp: PipewireLink[] = library.getLinks();
  return temp.filter(link => link.id);
}

export function getPorts(): PipewirePort[] {
  const temp: PipewirePort[] = library.getPorts();
  return temp.filter(port => port.id);
}

export function getNodes(): PipewireNode[] {
  const temp: PipewireNode[] = library.getNodes();
  return temp.filter(node => node.id);
}

export function getOutputNodes(): PipewireNode[] {
  const temp: PipewireNode[] = library.getOutputNodes();
  return temp.filter(output => output.id);
}

export function getInputNodes(): PipewireNode[] {
  const temp = library.getInputNodes();
  return temp.filter(input => input.id);
}

export function linkNodesNameToId(nodeName: string, nodeId: number) {
  library.linkNodesNameToId(nodeName, nodeId);
}

export function unlinkNodesNameToId(nodeName: string, nodeId: number) {
  library.unlinkNodesNameToId(nodeName, nodeId);
}

export function linkPorts(inputPortId: number, outputPortId: number) {
  library.linkPorts(inputPortId, outputPortId);
}

export function unlinkPorts(inputPortId: number, outputPortId: number) {
  library.unlinkPorts(inputPortId, outputPortId);
}

export function getInputNodesName(): string[] {
  return getInputNodes().map(input => input.name);
}

export function getOutputNodesName(): string[] {
  return getOutputNodes().map(output => output.name);
}

export function waitForNewNode(nodeName: string, direction?: NodeDirection, timeout?: number): Promise<PipewireNode> {
  return library.waitForNewNode(nodeName, direction ?? "Both", timeout ?? 5000);
}
