// Typescript definitions for node-pipewire
interface PipewirePort {
  id: number;
  permissions: number;
  props: Record<string, string>;
  node_id: number;
  name: string;
  direction: string;
}

interface PipewireNode {
  id: number;
  permissions: number;
  props: Record<string, string>;
  name: string;
  node_direction: string;
  node_type: string;
  ports: PipewirePort[];
}

interface PipewireLink {
  id: number;
  permissions: number;
  props: Record<string, string>;
  input_node_id: number;
  input_port_id: number;
  output_node_id: number;
  output_port_id: number;
}

interface PipewireClient {
  id: number;
  permissions: number;
  pid: number;
  application_name: string;
  props: Record<string, string>;
}

type NodeDirection = "Input" | "Output" | "Both";

// Surround is not yet implemented in the library
type AudioPosition = "FL" | "FR";

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

export function getClients(): PipewireClient[] {
  const temp: PipewireClient[] = library.getClients();
  return temp.filter(client => client.id);
}

export function getOutputNodes(): PipewireNode[] {
  const temp: PipewireNode[] = library.getOutputNodes();
  return temp.filter(output => output.id);
}

export function getInputNodes(): PipewireNode[] {
  const temp = library.getInputNodes();
  return temp.filter(input => input.id);
}

export function linkNodesNameToId(nodeName: string, nodeId: number, permanent = true) {
  library.linkNodesNameToId(nodeName, nodeId, permanent);
}

export function unlinkNodesNameToId(nodeName: string, nodeId: number) {
  library.unlinkNodesNameToId(nodeName, nodeId);
}

export function linkPorts(inputPortId: number, outputPortId: number, permanent = true) {
  library.linkPorts(inputPortId, outputPortId, permanent);
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

export function createSource(newSourceName: string, audioPositions: AudioPosition[], permanent = false) {
  return library.createSource(newSourceName, audioPositions.join(","), audioPositions.length, permanent);
}

export function createSink(newSinkName: string, audioPositions: AudioPosition[], permanent = false) {
  return library.createSink(newSinkName, audioPositions.join(","), audioPositions.length, permanent);
}

export function destroyObject(id: number) {
  return library.destroyObject(id);
}