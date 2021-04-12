# ggg

Gohla's game/graphics garage/garden/gadgetry/graveyard

## Prerequisites

### MacOS

Install the Vulkan SDK with homebrew:

```shell
brew tap apenngrace/homebrew-vulkan
brew install vulkan-sdk
```

## Building

```shell
cargo build -Z unstable-options --profile=fastdev
```

## Running

```shell
cargo run --bin cubes -Z unstable-options --profile=fastdev
```
