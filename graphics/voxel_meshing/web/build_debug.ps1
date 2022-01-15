Push-Location -Path "$PSScriptRoot/../../../"
try {
  cargo build --package voxel_meshing --target wasm32-unknown-unknown --target-dir target_wasm
  wasm-bindgen --out-dir graphics/voxel_meshing/web --target web --no-typescript --debug target_wasm/wasm32-unknown-unknown/debug/voxel_meshing.wasm
} finally {
  Pop-Location
}
