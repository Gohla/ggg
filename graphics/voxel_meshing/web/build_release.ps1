Push-Location -Path "$PSScriptRoot/../../../"
try {
  cargo build --package voxel_meshing --target wasm32-unknown-unknown --target-dir target_wasm --release
  wasm-bindgen --out-dir graphics/voxel_meshing/web --target web --no-typescript target_wasm/wasm32-unknown-unknown/release/voxel_meshing.wasm
} finally {
  Pop-Location
}
