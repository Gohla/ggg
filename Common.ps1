function Invoke-Wasm-Bindgen {
  param (
    [String]$Package,
    [String]$Binary = $null,
    [Boolean]$Debug = $false
  )

  Push-Location -Path "$PSScriptRoot"
  try {
    $BinPart = if($Binary -ne $null) { "--bin $Binary" } else { "" }
    $ReleasePart = if($Debug) { "" } else { "--release" }
    Invoke-Expression "cargo build --package $Package $BinPart --target wasm32-unknown-unknown --target-dir target_wasm $ReleasePart"

    $OutDir = if($Binary -ne $null) { "$Package/src/bin/$Binary" } else { "$Package" }
    $TargetSubdir = if($Debug) { "debug" } else { "release" }
    $WasmFileName = if($Binary -ne $null) { $Binary } else { $Package }
    Invoke-Expression "wasm-bindgen --out-dir $OutDir/web/wasm_out --target web --no-typescript target_wasm/wasm32-unknown-unknown/$TargetSubdir/$WasmFileName.wasm"
  } finally {
    Pop-Location
  }
}
