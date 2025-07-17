A simple framework for 2d pixel games.

**Very early release ;)**

## Supported platforms

- Windows
- Linux
- Wasm
- Android

## Features

- simple API for rapid prototyping
- stack based scene management (main menu, game, pause etc.)
- asset management incl. hot-reloading (dev builds) and embedding (release builds)
- sprite rendering with batching and z-sorting
- sprite atlases
- custom 2d meshes
- 2d point lights
- normal maps
- switchable 2d cameras
- pixel perfect rendering pass
- custom sprite shaders (no custom uniform buffers though)
- postprocessing shaders (also no custom uniforms)
- limited text rendering (currently only ASCII table style bitmap fonts)
- basic audio

## Examples

You can find some basic usage and simple game examples in the main crate folder:
<https://github.com/maciekglowka/rogalik/tree/main/crates/rogalik/examples>

## Todo

- proper font rendering (ttf)
- documentation ;)
- examples
- `fixed_update` method

## Games created with Rogalik

- <https://github.com/maciekglowka/grimvaders>
- <https://github.com/maciekglowka/tower-rl>
- <https://github.com/maciekglowka/fish_bots>
- <https://github.com/maciekglowka/ugh-like>
