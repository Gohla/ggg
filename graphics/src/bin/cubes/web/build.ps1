param (
  [Boolean]$Debug = $false
)

. .\..\..\..\..\..\Common.ps1

Invoke-Wasm-Bindgen -Package "graphics" -Binary "cubes" -Debug $Debug
