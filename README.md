A simple framework for 2d pixel games.

## Supported platforms

- Windows
- Linux
- Wasm
- Android

## Features

- simple API for rapid prototyping
- stack based scene management (main menu, game, pause etc.)
- asset management incl. hot-reloading (dev tools enabled) and embedding (default)
- sprite rendering with batching and z-sorting
- sprite atlases
- custom 2d meshes
- 2d point lights
- normal maps
- switchable 2d cameras
- pixel perfect rendering pass
- custom sprite shaders (no custom uniform buffers though)
- postprocessing shaders (also no custom uniforms)
- text rendering (bitmap and ttf fonts) 
- basic audio
- remote control / screenshot (experimental)
- dev mode video recording (experimental)

## Dev tools

To enable dev-tools set Rust cfg `dev_tools`, e.g. via setting `RUSTFLAGS` var
when running the game.

```ignore
RUSTFLAGS="--cfg mock" cargo run
```

By default this only enables assets hot reloading.

Other dev tools have to be enabled by setting appropriate Cargo features
as well:

- `remote` - enables remote websocket control
- `capture` - enables recording

## Examples

Examples are provided as separate crates in the `examples` directory.

In order to run them execute e.g.:

```ignore
cargo run --bin sprite-atlas
```

## Games created with Rogalik

- <https://github.com/maciekglowka/grimvaders>
- <https://github.com/maciekglowka/tower-rl>
- <https://github.com/maciekglowka/fish_bots>
- <https://github.com/maciekglowka/ugh-like>
