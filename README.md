# Gatekeeper Door Client

Door lock client for the Gatekeeper access control system.

# Building

To download and build this project on a development machine, run the following:

```
git clone https://github.com/stevenmirabito/gatekeeper-door.git
cd gatekeeper-door
cargo build
```

To build this project for the target hardware, you'll need to cross-compile it
using the [`cross`] tool. To use it, first install it using

[`cross`]: https://github.com/rust-embedded/cross

```
cargo install cross
```

Then, you can use `cross` as an almost-drop-in-replacement for cargo to
build the project.

```
cross build --target=armv7-unknown-linux-gnueabihf
```

Cross uses Docker under the hood to put together a container environment with
the correct cross-compilers needed. For this project, we also specify
how to install third-party dependencies in this cross-compiled environment by
placing those installation steps in the `Dockerfile` which `cross` uses.
