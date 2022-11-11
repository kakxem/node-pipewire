# node-pipewire

## Requirements

As said, this module is a native Node.js module. So, if you want to use it, you need to have to compile it. For that, you need to have the following dependencies installed:

- [Node.js](https://nodejs.org/en/)
- [Rust](https://www.rust-lang.org/)
- [Pipewire](https://pipewire.org/)

Many distributions have pipewire as audio server by default, but maybe your distro doesn't. You can check if you have pipewire installed by running the following command:

```bash
$ pactl info
```

If that's the case, you will need to install it manually.
Fedora (34 and above) and Ubuntu (22.10 and above) has pipewire as default audio server, so you don't need to install it manually. 

Depending on your linux distribution, you may need to install some other dependencies to compile the module.

Fedora:

```bash
$ sudo dnf install pipewire-devel clang
```

Ubuntu:

```bash
$ sudo apt install build-essential libpipewire-0.3-dev
```
  
If you are using a different distribution, you will need to find the equivalent packages. (If you find them, please open a PR to add them to this README)

## Installation

First of all, we'll need to install the module:

```sh
$ npm install node-pipewire
```

Then, we'll need to compile the rust code to generate the native module:

```sh
$ cd node_modules/node-pipewire
$ npm install 
```

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
createPwThread()

//Returns a list of nodes.
getNodes()

//Returns a list of ports.
getPorts()

//Returns a list of links.
getLinks()

//Returns a list of output nodes.
getOutputNodes()

//Returns a list of input nodes.
getInputNodes()

//Returns a list of name of input nodes.
getInputNodesName()

//Returns a list of name of output nodes.
getOutputNodesName()

//Links two ports.
linkPorts(inputPortId, outputPortId)

//Unlinks two ports.
unlinkPorts(inputPortId, outputPortId)

//Link all nodes that have the name `nodeName` to the node with the id `nodeId`.
linkNodesNameToId(nodeName, nodeId)

//Unlink all nodes that have the name `nodeName` to the node with the id `nodeId`.
unlinkNodesNameToId(nodeName, nodeId)
```

## Development

This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).

Clone the repository:
  
```sh
  $ git clone https://github.com/kakxem/node-pipewire.git
  $ cd node-pipewire
```

### Installing node-pipewire

Installing node-pipewire requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

You can install the project with npm. In the project directory, run:

```sh
$ npm install
```

This fully installs the project, including installing any dependencies and running the build.

### Building node-pipewire

If you have already installed the project and only want to run the build, run:

```sh
$ npm run build
```

This command uses the [cargo-cp-artifact](https://github.com/neon-bindings/cargo-cp-artifact) utility to run the Rust build and copy the built library into `./index.node`.

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

Runs the unit tests by calling `cargo test`. You can learn more about [adding tests to your Rust code](https://doc.rust-lang.org/book/ch11-01-writing-tests.html) from the [Rust book](https://doc.rust-lang.org/book/).

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
|   └── node/
|       ├── index.ts
|       └── types.ts
├── build/
|   ├── (.js, .d.ts, .js.map files)
|   └── index.node
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

##### src/node/

The directory tree containing the TypeScript source code for the project.

###### src/node/index.ts

The TypeScript module's main module.

###### src/node/types.ts

The TypeScript module's type definitions.

#### build/

The directory tree containing the built JS/TS files and the native module compiled.

##### build/(.js, .d.ts, .js.map files)

The built JavaScript and TypeScript files.

##### build/index.node

The Node addon—i.e., a binary Node module—generated by building the project. This is the main module for this package, as dictated by the `"main"` key in `package.json`.

Under the hood, a [Node addon](https://nodejs.org/api/addons.html) is a [dynamically-linked shared object](https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries). The `"build"` script produces this file by copying it from within the `target/` directory, which is where the Rust build produces the shared object.

#### target/

Binary artifacts generated by the Rust build.

### Learn More

To learn more about Neon, see the [Neon documentation](https://neon-bindings.com).

To learn more about Rust, see the [Rust documentation](https://www.rust-lang.org).

To learn more about Node, see the [Node documentation](https://nodejs.org).
