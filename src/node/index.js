"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getOutputNodesName = exports.getInputNodesName = exports.unlinkPorts = exports.linkPorts = exports.unlinkNodesNameToId = exports.linkNodesNameToId = exports.getInputNodes = exports.getOutputNodes = exports.getNodes = exports.getPorts = exports.getLinks = exports.closePwThread = exports.createPwThread = void 0;
const library = require("./../../index.node");
function createPwThread() {
    return library.createPwThread();
}
exports.createPwThread = createPwThread;
function closePwThread() {
    return library.closePwThread();
}
exports.closePwThread = closePwThread;
function getLinks() {
    let temp = library.getLinks();
    let links = [];
    for (let i = 0; i < temp.length; i++) {
        if (temp[i].id) {
            links.push(temp[i]);
        }
    }
    return links;
}
exports.getLinks = getLinks;
function getPorts() {
    let temp = library.getPorts();
    let ports = [];
    for (let i = 0; i < temp.length; i++) {
        if (temp[i].id) {
            ports.push(temp[i]);
        }
    }
    return ports;
}
exports.getPorts = getPorts;
function getNodes() {
    let temp = library.getNodes();
    let nodes = [];
    for (let i = 0; i < temp.length; i++) {
        if (temp[i].id) {
            nodes.push(temp[i]);
        }
    }
    return nodes;
}
exports.getNodes = getNodes;
function getOutputNodes() {
    let temp = library.getOutputNodes();
    let nodes = [];
    for (let i = 0; i < temp.length; i++) {
        if (temp[i].id) {
            nodes.push(temp[i]);
        }
    }
    return nodes;
}
exports.getOutputNodes = getOutputNodes;
function getInputNodes() {
    let temp = library.getOutputNodes();
    let nodes = [];
    for (let i = 0; i < temp.length; i++) {
        if (temp[i].id) {
            nodes.push(temp[i]);
        }
    }
    return nodes;
}
exports.getInputNodes = getInputNodes;
function linkNodesNameToId(nodeName, nodeId) {
    return library.linkNodesNameToId(nodeName, nodeId);
}
exports.linkNodesNameToId = linkNodesNameToId;
function unlinkNodesNameToId(nodeName, nodeId) {
    return library.unlinkNodesNameToId(nodeName, nodeId);
}
exports.unlinkNodesNameToId = unlinkNodesNameToId;
function linkPorts(inputPortId, outputPortId) {
    return library.linkPorts(inputPortId, outputPortId);
}
exports.linkPorts = linkPorts;
function unlinkPorts(inputPortId, outputPortId) {
    return library.unlinkPorts(inputPortId, outputPortId);
}
exports.unlinkPorts = unlinkPorts;
function getInputNodesName() {
    const temp = getInputNodes();
    const nodes = [];
    for (let i = 0; i < temp.length; i++) {
        nodes.push(temp[i].name);
    }
    return nodes;
}
exports.getInputNodesName = getInputNodesName;
function getOutputNodesName() {
    const temp = getOutputNodes();
    const nodes = [];
    for (let i = 0; i < temp.length; i++) {
        nodes.push(temp[i].name);
    }
    return nodes;
}
exports.getOutputNodesName = getOutputNodesName;
