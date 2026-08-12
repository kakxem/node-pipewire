# node-pipewire

<table>
  <tr>
      <td> ⚠️ </td>
      <td>
          <p><b>Please note that the type definitions will no longer be included in this repository.</b></p>
          <p>As a result, you'll need to manually <b>install @types/node-pipewire</b>.</p>
          <p>This change is due to the fact that if the library is being used in a multi-platform project, it would likely be listed in 'optionalDependencies'. If the library is absent, this ensures the project won't be missing the necessary type definitions.</p>
          <p>Type definitions are available in the <a href="https://www.npmjs.com/package/@types/node-pipewire">npm registry</a>.</p>
      </td>
  </tr>
</table>

## Requirements

This package is a native Node.js module for Linux and requires:

- [Node.js](https://nodejs.org/en/) 18 or newer
- The [PipeWire](https://pipewire.org/) runtime library (`libpipewire-0.3.so.0`)

### Prebuilt binary compatibility

Prebuilt Node-API 6 binaries are available for Linux x86-64 and ARM64 as
`napi-v6-linux-x64.tar.gz` and `napi-v6-linux-arm64.tar.gz`. Node-API
provides a stable ABI across Node.js releases, allowing each binary to
work with every officially supported Node.js version. This package
supports Node.js 18 and newer.

The prebuilt binaries target glibc 2.34 or newer. Systems using an older
glibc version, musl, or another CPU architecture can build the addon
from source.

If no compatible prebuild is available, install from source. This additionally requires Rust, Clang, a C toolchain, and the PipeWire development headers. You can explicitly request a source build with:

```sh
npm install node-pipewire --build-from-source
```

Many distributions have pipewire as audio server by default, but maybe your distro doesn't. You can check if you have pipewire installed by running the following command:

```bash
pactl info
```

If that's the case, you will need to install it manually.
Fedora (34 and above) and Ubuntu (22.10 and above) has pipewire as default audio server, so you don't need to install it manually.

Depending on your linux distribution, you may need to install some other dependencies to compile the module.

Fedora:

```bash
sudo dnf install pipewire-devel clang
```

Ubuntu:

```bash
sudo apt install build-essential libpipewire-0.3-dev
```
  
If you are using a different distribution, you will need to find the equivalent packages. (If you find them, please open a PR to add them to this README)

## Installation

First of all, we'll need to install the module:

```sh
npm install node-pipewire
```

On a supported Linux x64 or ARM64 system, npm downloads the prebuilt
addon. If the prebuild cannot be downloaded, npm falls back to compiling
from source. If you have any problem, please open an issue.

## Usage

```js
import { createPwThread, getNodes } from 'node-pipewire';

async function main() {
  createPwThread();

  await new Promise((resolve) => setTimeout(resolve, 1000));

  const nodes = await getNodes();
  console.log(nodes);
}

main();
```

## API

```ts
//Creates a thread that runs the pipewire loop.
createPwThread(enableDebug?: boolean)

//Returns a list of nodes.
getNodes() 

//Returns a list of ports.
getPorts()

//Returns a list of links.
getLinks()

//Returns a list of clients.
getClients()

//Returns a list of output nodes.
getOutputNodes()

//Returns a list of input nodes.
getInputNodes()

//Returns a list of name of input nodes.
getInputNodesName()

//Returns a list of name of output nodes.
getOutputNodesName()

//Links two ports. If permanent is false, the link will become disconnected after the PwThread closes.
linkPorts(inputPortId: number, outputPortId, number, permanent = true)

//Unlinks two ports.
unlinkPorts(inputPortId: number, outputPortId: number)

//Link all nodes that have the name `nodeName` to the node with the id `nodeId`. If permanent is false, the link will become disconnected after the PwThread closes.
linkNodesNameToId(nodeName: string, nodeId: number, permanent = true)

//Unlink all nodes that have the name `nodeName` to the node with the id `nodeId`.
unlinkNodesNameToId(nodeName: string, nodeId: number)

//Wait for a node to be created.
waitForNewNode(nodeName: string, direction?: 'Input' | 'Output' | 'Both', timeout?: number)

//Create a new source. If permanent is false, the node will be deleted after the PwThread closes.
//Sources will be created with `node-pipewire:` in front of the name, but the nickname will be the string passed.
//Passing an empty audioPositions array will result in an error being thrown.
createSource(sourceName: string, audioPositions: AudioPosition[], permanent = false)

//Create a new sink. If permanent is false, the node will be deleted after the PwThread closes.
//Sinks will be created with `node-pipewire:` in front of the name, but the nickname will be the string passed.
//Passing an empty audioPositions array will result in an error being thrown.
createSink(sourceName: string, audioPositions: AudioPosition[], permanent = false)

//Destroy an object. This will only succeed if ID represents a link, or a sink/source created by node-pipewire.
//Destroying a node may crash applications referencing that node.
destroyObject(id: number);
```

## Development

This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).

If Nix is available, use the provided development flake to get the
required build and test dependencies:

```sh
nix develop
```

Clone the repository:
  
```sh
  git clone https://github.com/kakxem/node-pipewire.git
  cd node-pipewire
```

### Installing node-pipewire

Installing node-pipewire requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

You can install the project with npm. In the project directory, run:

```sh
npm install
```

This fully installs the project, including installing any dependencies and running the build.

### Building node-pipewire

If you have already installed the project and only want to run the build, run:

```sh
npm run build
```

This command uses the [cargo-cp-artifact](https://github.com/neon-bindings/cargo-cp-artifact) utility to run the Rust build and copy the built library into `dist/binding/napi-v6/index.node`.

### Exploring node-pipewire

After building node-pipewire, you can explore its exports at the Node REPL:

```sh
$ npm install
$ node
> const pipewire = require('.')
> pipewire.createPwThread()
> console.log(pipewire.getNodes())
"
[
  ..
]
"
```

You can also create a new file in the project directory and make your own experiments:

```js
const test = require('.');

test.createPwThread();

setTimeout(() => {
  console.log(test.getNodes());
}, 1000);
```

### Available Scripts

In the project directory, you can run:

#### `npm install`

Installs the project, including running `npm run build`.

#### `npm build`

Builds the Node addon (`index.node`) from source and transpile TS file to JS.

Additional [`cargo build`](https://doc.rust-lang.org/cargo/commands/cargo-build.html) arguments may be passed to `npm build` and `npm build-*` commands. For example, to enable a [cargo feature](https://doc.rust-lang.org/cargo/reference/features.html):

```
npm run build -- --feature=beetle
```

#### `npm build-debug`

Alias for `npm build`.

#### `npm build-release`

Same as [`npm build`](#npm-build) but, builds the module with the [`release`](https://doc.rust-lang.org/cargo/reference/profiles.html#release) profile. Release builds will compile slower, but run faster.

#### `npm test`

Runs the Rust integration tests serially because they share PipeWire resources.

### Project Layout

The directory structure of this project is:

```
node-pipewire/
├── Cargo.toml
├── README.md
├── package.json
├── src/
|   ├── lib.rs
|   ├── pipewire_thread.rs
|   ├── proxy.rs
|   └── node/
|       ├── index.ts
|       └── types.ts
├── dist/
|   ├── (.js, .d.ts, .js.map files)
|   └── binding/
|       └── napi-v6/
|           └── index.node
└── target/
```

#### Cargo.toml

The Cargo [manifest file](https://doc.rust-lang.org/cargo/reference/manifest.html), which informs the `cargo` command.

#### README.md

This file.

#### package.json

The npm [manifest file](https://docs.npmjs.com/cli/v7/configuring-npm/package-json), which informs the `npm` command.

#### src/

The directory tree containing the source code for the project.

##### src/lib.rs

The Rust library's main module.

##### src/pipewire_thread.rs

The Rust code for the pipewire thread.

##### src/proxy.rs

The Rust wrapper for PipeWire proxy objects.

##### src/node/

The directory tree containing the TypeScript source code for the project.

###### src/node/index.ts

The TypeScript module's main module.

###### src/node/types.ts

The TypeScript module's type definitions.

#### dist/

The directory tree containing the built JS/TS files and the native module compiled.

##### dist/(.js, .d.ts, .js.map files)

The built JavaScript and TypeScript files.

##### dist/binding/napi-v6/index.node

The native Node addon generated by the Rust build and loaded by
`dist/index.js`.

Under the hood, a [Node addon](https://nodejs.org/api/addons.html) is a [dynamically-linked shared object](https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries). The `"build"` script produces this file by copying it from within the `target/` directory, which is where the Rust build produces the shared object.

#### target/

Binary artifacts generated by the Rust build.

### Learn More

To learn more about Neon, see the [Neon documentation](https://neon-bindings.com).

To learn more about Rust, see the [Rust documentation](https://www.rust-lang.org).

To learn more about Node, see the [Node documentation](https://nodejs.org).

### Contribution

If you are interested in contributing to this project, please read the [CONTRIBUTING](CONTRIBUTING.md) file for more information.

Thank you for your interest in contributing!
