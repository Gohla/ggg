Push-Location -Path "$PSScriptRoot/../../../"
try {
  cargo build --package graphics --bin voxel_planets --target wasm32-unknown-unknown --target-dir target_wasm
  wasm-bindgen --out-dir graphics/bin/voxel_planets/web --target web --no-typescript --debug target_wasm/wasm32-unknown-unknown/debug/voxel_planets.wasm
} finally {
  Pop-Location
}
