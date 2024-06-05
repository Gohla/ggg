# ggg

Gohla's game/graphics/gadget garage/garden/gadgetry/graveyard

## Prerequisites

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

## Building

```shell
cargo build
```

## Running

```shell
cargo run --bin cubes
```
