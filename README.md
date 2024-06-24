# ggg

Gohla's game/graphics/gadget garage/garden/gadgetry

Developed in [Rust](https://www.rust-lang.org/) using [wgpu](https://github.com/gfx-rs/wgpu) for rendering.

## Prerequisites

A recent version of Rust is required. See [Rust's installation instructions](https://www.rust-lang.org/tools/install).

## Running

Main graphics demos:

* Voxel planetoid renderer: `cargo run --bin voxel_planets`
  * Note: surface nets mesher disabled due to bug in LoD stitching.
  * Note: transvoxel mesher doesn't correctly extract border regions.
* Marching cubes visualization: `cargo run --bin marching_cubes`
* Rendering many cubes demo: `cargo run --bin cubes`

Small examples:

* Triangle: `cargo run --bin triangle`
* Quads: `cargo run --bin quads`

Currently **broken** graphics demos:

* Ray tracing in one weekend, in a GLSL fragment shader: `cargo run --bin ray_tracing`
  * Renders only the "sky". Probably broken due to `naga` (wpgu's shader compiler) not compiling the shader properly, or at least not the way `shaderc` was compiling it.
* Surface nets visualization: `cargo run --bin surface_nets` 
  * Crashes due to a LoD stitching bug

By default, Rust runs with a debug/development profile. To run with full optimization and use optimized CPU instructions for you CPU, run like this:

```shell
RUSTFLAGS=-Ctarget-cpu=native cargo run --bin voxel_planets --release
```

## Structure

* core: core libraries
  * `common`: common data structures and helpers
  * `os`: operating system interface, providing logging, input handling, etc.
  * `gfx`: wgpu utilities, GPU buffer utilities, and common graphics code such as camera projection.
  * `gfxc`: shader compiler
  * `egui_integration`: integrate `egui` with `os` and `gfx`, providing an immediate mode GUI.
  * `gui`: common gui code and widgets
  * `app`: application framework taking care of setting up `os`, `gfx` and `egui_integration`, and taking care of running the game loop.
    * All demos in this repository implement the `App` trait so the demos can focus on the actual functionality.
  * `job_queue`: parallel job queue with support for dependencies and referencing/caching computed data
  * `voxel`: voxels, level of detail (Lod), voxel meshing, and procedural generation via noise.
    * Voxel meshing implementations: marching cubes, transvoxel (incomplete), and naive surface nets.
    * Level of detail stitching between different LoD levels is still incomplete/buggy.
* graphics/src/bin: graphics demos
  * `triangle`: render a single triangle, every renderer needs this :)
  * `quads`: render some quads with a texture
  * `cubes`: render many cubes efficiently
  * `marching_cubes`: marching cubes visualization
  * `surface_nets`: surface nets visualization
  * `voxel_planets`: voxel planetoid renderer
* gadgets/src/bin: gadgets
  * `job_queue`: example on using the job queue

## Vulkan validation layers

When using the Vulkan backend of wgpu, and you want Vulkan's validation layers to run, you need to install the Vulkan SDK.
Install the [Vulkan SDK using their own installer](https://vulkan.lunarg.com/sdk/home), or use a package manager:

### Windows

Install the Vulkan SDK. This can be done via scoop:

```powershell
scoop install main/vulkan
```

Then follow the instructions ("Allow vulkan applications to find VK layers provided by Khronos, run ...") after installing to make Vulkan validation layers work.

### MacOS

Install the Vulkan SDK with homebrew:

```shell
brew tap apenngrace/homebrew-vulkan
brew install vulkan-sdk
```
