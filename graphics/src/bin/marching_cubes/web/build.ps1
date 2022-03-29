param (
  [Boolean]$Debug = $false
)

. .\..\..\..\..\..\Common.ps1

Invoke-Wasm-Bindgen -Package "graphics" -Binary "marching_cubes" -Debug $Debug
