param (
  [Boolean]$Debug = $false
)

. .\..\..\..\..\..\Common.ps1

Invoke-Wasm-Bindgen -Package "graphics" -Binary "triangle" -Debug $Debug
