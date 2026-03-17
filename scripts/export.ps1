param(
  [ValidateSet("Release", "Both")]
  [string]$Build = "Release",

  [string]$GodotExe = "godot",
  [string]$PresetName = "Windows Desktop",

  [string]$OutDir = "export",
  [string]$ExeName = "game.exe",

  [switch]$ForceCreateExportPreset,
  [switch]$IncludePdb,
  [switch]$NoRecoveryMode
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Assert-CommandExists([string]$CommandName) {
  $cmd = Get-Command $CommandName -ErrorAction SilentlyContinue
  if (-not $cmd) {
    throw "Command not found: '$CommandName'. Ensure it is installed and on PATH."
  }
}

function Assert-LastExitCode([string]$Action) {
  if ($LASTEXITCODE -ne 0) {
    throw "$Action failed with exit code $LASTEXITCODE."
  }
}

function Resolve-CommandPath([string]$CommandName) {
  $cmd = Get-Command $CommandName -ErrorAction SilentlyContinue
  if (-not $cmd) {
    throw "Command not found: '$CommandName'. Ensure it is installed and on PATH."
  }

  if ($cmd.Path) {
    return $cmd.Path
  }

  return $cmd.Source
}

function Test-IsWindows() {
  return [System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT
}

function Resolve-GodotExecutable([string]$RequestedExe) {
  $resolvedExe = Resolve-CommandPath $RequestedExe
  if (-not (Test-IsWindows)) {
    return $resolvedExe
  }

  $dir = Split-Path -Parent $resolvedExe
  $base = [System.IO.Path]::GetFileNameWithoutExtension($resolvedExe)
  $ext = [System.IO.Path]::GetExtension($resolvedExe)
  if ($base -match '(?i)_console$') {
    return $resolvedExe
  }

  $consoleSibling = Join-Path $dir ($base + "_console" + $ext)
  if (Test-Path -LiteralPath $consoleSibling) {
    return $consoleSibling
  }

  return $resolvedExe
}

function Wait-ForPath([string]$Path, [int]$TimeoutSeconds = 30, [int]$PollIntervalMilliseconds = 200) {
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    if (Test-Path -LiteralPath $Path) {
      return $true
    }
    Start-Sleep -Milliseconds $PollIntervalMilliseconds
  }

  return (Test-Path -LiteralPath $Path)
}

function Normalize-GodotExtensionList([string]$GodotProjectDir) {
  $godotCacheDir = Join-Path $GodotProjectDir ".godot"
  $extensionListPath = Join-Path $godotCacheDir "extension_list.cfg"
  $defaultExt = "res://rust.gdextension"

  if (-not (Test-Path $extensionListPath)) {
    return
  }

  $raw = Get-Content -Raw -LiteralPath $extensionListPath
  $lines = $raw -split "`r?`n" | ForEach-Object { $_.Trim() } | Where-Object { $_.Length -gt 0 }

  $kept = [System.Collections.Generic.List[string]]::new()
  foreach ($line in $lines) {
    if ($line -notmatch '^res://') { continue }
    $rel = $line.Substring(6)
    $fsPath = Join-Path $GodotProjectDir $rel
    if (Test-Path $fsPath) {
      [void]$kept.Add($line)
    }
  }

  $defaultRel = $defaultExt.Substring(6)
  if (Test-Path (Join-Path $GodotProjectDir $defaultRel)) {
    if (-not ($kept -contains $defaultExt)) {
      $kept.Insert(0, $defaultExt)
    }
  }

  Set-Content -LiteralPath $extensionListPath -Value ($kept -join "`n") -Encoding UTF8
}

function Ensure-ExportPresets([string]$GodotProjectDir, [string]$PresetName, [bool]$Force) {
  $exportPresetsPath = Join-Path $GodotProjectDir "export_presets.cfg"

  $defaultOptions = @(
    'binary_format/architecture="x86_64"'
    'binary_format/embed_pck=false'
  )

  if (-not (Test-Path $exportPresetsPath)) {
    $content = @"
[preset.0]
name="$PresetName"
platform="Windows Desktop"
runnable=true
dedicated_server=false
custom_features=""
export_filter="all_resources"
include_filter=""
exclude_filter=""
export_path=""
encryption_include_filters=""
encryption_exclude_filters=""
encrypt_pck=false
encrypt_directory=false

[preset.0.options]
$($defaultOptions -join "`r`n")
"@
    New-Item -ItemType Directory -Force -Path $GodotProjectDir | Out-Null
    Set-Content -LiteralPath $exportPresetsPath -Value $content -Encoding UTF8
    return
  }

  $existing = Get-Content -Raw -LiteralPath $exportPresetsPath

  $lines = $existing -split "`r?`n"
  $maxIndex = -1
  $currentIndex = $null
  $foundIndex = $null

  foreach ($line in $lines) {
    if ($line -match '^\[preset\.(\d+)\]') {
      $currentIndex = [int]$Matches[1]
      if ($currentIndex -gt $maxIndex) { $maxIndex = $currentIndex }
      continue
    }
    if ($currentIndex -ne $null -and $line -eq "name=""$PresetName""") {
      $foundIndex = $currentIndex
    }
  }

  $resultLines = [System.Collections.Generic.List[string]]::new()
  foreach ($l in $lines) { [void]$resultLines.Add($l) }

  $ensureOptionsForIndex = {
    param([int]$Index)
    $header = "[preset.$Index.options]"

    $headerLine = -1
    for ($i = 0; $i -lt $resultLines.Count; $i++) {
      if ($resultLines[$i] -eq $header) { $headerLine = $i; break }
    }

    if ($headerLine -lt 0) {
      [void]$resultLines.Add("")
      [void]$resultLines.Add($header)
      foreach ($opt in $defaultOptions) { [void]$resultLines.Add($opt) }
      return
    }

    $hasKey = $false
    for ($j = $headerLine + 1; $j -lt $resultLines.Count; $j++) {
      $line = $resultLines[$j]
      if ($line -match '^\[.+\]') { break }
      if ($line.Trim().Length -eq 0) { continue }
      $hasKey = $true
      break
    }

    if (-not $hasKey) {
      $insertAt = $headerLine + 1
      foreach ($opt in $defaultOptions) {
        $resultLines.Insert($insertAt, $opt)
        $insertAt++
      }
    }
  }

  if ($foundIndex -ne $null) {
    & $ensureOptionsForIndex $foundIndex
    Set-Content -LiteralPath $exportPresetsPath -Value ($resultLines -join "`r`n") -Encoding UTF8
    return
  }

  if (-not $Force) {
    throw "Export preset '$PresetName' not found in $exportPresetsPath. Create it in the Godot editor, or re-run with -ForceCreateExportPreset to append it."
  }

  $newIndex = $maxIndex + 1
  [void]$resultLines.Add("")
  [void]$resultLines.Add("[preset.$newIndex]")
  [void]$resultLines.Add("name=""$PresetName""")
  [void]$resultLines.Add('platform="Windows Desktop"')
  [void]$resultLines.Add("runnable=true")
  [void]$resultLines.Add("dedicated_server=false")
  [void]$resultLines.Add('custom_features=""')
  [void]$resultLines.Add('export_filter="all_resources"')
  [void]$resultLines.Add('include_filter=""')
  [void]$resultLines.Add('exclude_filter=""')
  [void]$resultLines.Add('export_path=""')
  [void]$resultLines.Add('encryption_include_filters=""')
  [void]$resultLines.Add('encryption_exclude_filters=""')
  [void]$resultLines.Add("encrypt_pck=false")
  [void]$resultLines.Add("encrypt_directory=false")

  & $ensureOptionsForIndex $newIndex
  Set-Content -LiteralPath $exportPresetsPath -Value ($resultLines -join "`r`n") -Encoding UTF8
}

Assert-CommandExists "cargo"
$resolvedGodotExe = Resolve-GodotExecutable $GodotExe

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$rustDir = Join-Path $repoRoot "rust"
$godotDir = Join-Path $repoRoot "godot"
$outDirAbs = Resolve-Path (Join-Path $repoRoot $OutDir) -ErrorAction SilentlyContinue
if (-not $outDirAbs) {
  New-Item -ItemType Directory -Force -Path (Join-Path $repoRoot $OutDir) | Out-Null
  $outDirAbs = Resolve-Path (Join-Path $repoRoot $OutDir)
}
$exportExeAbs = Join-Path $outDirAbs $ExeName

Write-Host "Using Godot executable: $resolvedGodotExe"

Write-Host "Building Rust GDExtension ($Build)..."
Push-Location $rustDir
try {
  & cargo build --release --locked
  Assert-LastExitCode "cargo build --release --locked"
  if ($Build -eq "Both") {
    & cargo build --locked
    Assert-LastExitCode "cargo build --locked"
  }
} finally {
  Pop-Location
}

Write-Host "Ensuring Godot export preset exists ($PresetName)..."
Ensure-ExportPresets -GodotProjectDir $godotDir -PresetName $PresetName -Force ([bool]$ForceCreateExportPreset)
Normalize-GodotExtensionList -GodotProjectDir $godotDir

$godotVersionOutput = & $resolvedGodotExe --version 2>$null
Assert-LastExitCode "$resolvedGodotExe --version"
$godotVersion = $godotVersionOutput | Select-Object -First 1
$templateVersion = $null
if ($godotVersion -match '^(\d+\.\d+(?:\.\d+)?\.[^\.]+)') {
  $templateVersion = $Matches[1]
}
$templatesRoot = Join-Path $env:APPDATA "Godot/export_templates"
if ($templateVersion) {
  $templatesDir = Join-Path $templatesRoot $templateVersion
  $winReleaseTemplate = Join-Path $templatesDir "windows_release_x86_64.exe"
  $winDebugTemplate = Join-Path $templatesDir "windows_debug_x86_64.exe"
  if (-not (Test-Path $winReleaseTemplate) -or -not (Test-Path $winDebugTemplate)) {
    throw @"
Missing Godot export templates for $templateVersion.
Install them via: Godot Editor -> Editor -> Manage Export Templates -> Download and Install.
Expected at: $templatesDir
"@
  }
}

try {
  Write-Host "Exporting with Godot..."
  Push-Location $repoRoot
  try {
    $godotArgs = @("--headless")
    if (-not $NoRecoveryMode) {
      $godotArgs += "--recovery-mode"
    }
    $godotArgs += @("--path", $godotDir, "--export-release", $PresetName, $exportExeAbs)
    & $resolvedGodotExe @godotArgs
    Assert-LastExitCode "$resolvedGodotExe $($godotArgs -join ' ')"
  } finally {
    Pop-Location
  }
} finally {
}

if (-not (Wait-ForPath -Path $exportExeAbs -TimeoutSeconds 30)) {
  throw "Export failed: output exe not found at $exportExeAbs"
}

# Ensure the extension DLL is available next to the exported exe (Godot may already copy it).
$exportRustDll = Join-Path $outDirAbs "rust.dll"
if (-not (Test-Path $exportRustDll)) {
  Copy-Item -LiteralPath (Join-Path $rustDir "target/release/rust.dll") -Destination $exportRustDll -Force
}
if ($IncludePdb) {
  $exportRustPdb = Join-Path $outDirAbs "rust.pdb"
  $srcPdb = Join-Path $rustDir "target/release/rust.pdb"
  if (Test-Path $srcPdb) {
    Copy-Item -LiteralPath $srcPdb -Destination $exportRustPdb -Force
  }
}

Write-Host "Done."
Write-Host "Output: $exportExeAbs"
Write-Host "GDExtension DLL (release): $exportRustDll"
Write-Host "Distribute the folder: $outDirAbs"
