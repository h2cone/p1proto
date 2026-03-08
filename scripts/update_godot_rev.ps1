param(
  [string]$RepoUrl = "https://github.com/godot-rust/gdext.git",
  [string]$Branch = "master",
  [switch]$DryRun,
  [switch]$SkipLockfile
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Assert-CommandExists([string]$CommandName) {
  $cmd = Get-Command $CommandName -ErrorAction SilentlyContinue
  if (-not $cmd) {
    throw "Command not found: '$CommandName'. Ensure it is installed and on PATH."
  }
}

function Try-GetCommand([string]$CommandName) {
  return Get-Command $CommandName -ErrorAction SilentlyContinue
}

function Get-GitHubApiCommitUrl([string]$RepositoryUrl, [string]$BranchName) {
  if ($RepositoryUrl -match '^https://github\.com/(?<owner>[^/]+)/(?<repo>[^/.]+?)(?:\.git)?/?$') {
    return "https://api.github.com/repos/$($Matches.owner)/$($Matches.repo)/commits/$BranchName"
  }

  if ($RepositoryUrl -match '^git@github\.com:(?<owner>[^/]+)/(?<repo>[^/.]+?)(?:\.git)?$') {
    return "https://api.github.com/repos/$($Matches.owner)/$($Matches.repo)/commits/$BranchName"
  }

  return $null
}

function Get-LatestGitRev([string]$RepositoryUrl, [string]$BranchName) {
  $refName = "refs/heads/$BranchName"
  $apiUrl = Get-GitHubApiCommitUrl -RepositoryUrl $RepositoryUrl -BranchName $BranchName

  if ($apiUrl) {
    try {
      $response = Invoke-RestMethod -Headers @{ 'User-Agent' = 'p1proto-update-script' } -Uri $apiUrl
      $rev = $response.sha
      if ($rev -match '^[0-9a-f]{40}$') {
        return $rev
      }
      throw "Unexpected rev returned by GitHub API: '$rev'"
    }
    catch {
      Write-Warning "GitHub API lookup failed, falling back to git ls-remote. $_"
    }
  }

  $git = Try-GetCommand "git"
  if ($git) {
    $hadNativePreference = Test-Path variable:PSNativeCommandUseErrorActionPreference
    if ($hadNativePreference) {
      $previousNativePreference = $PSNativeCommandUseErrorActionPreference
      $script:PSNativeCommandUseErrorActionPreference = $false
    }

    try {
      $output = & git ls-remote $RepositoryUrl $refName 2>$null
    } finally {
      if ($hadNativePreference) {
        $script:PSNativeCommandUseErrorActionPreference = $previousNativePreference
      }
    }

    if ((Test-Path variable:LASTEXITCODE) -and $LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($output)) {
      $rev = ($output -split "\s+")[0].Trim()
      if ($rev -match '^[0-9a-f]{40}$') {
        return $rev
      }
    }
  }

  throw "Could not resolve latest rev for '$RepositoryUrl' '$refName'."
}

function Get-GodotRevMatch([string]$CargoTomlPath) {
  $content = Get-Content -Raw -LiteralPath $CargoTomlPath
  $pattern = '(?ms)(godot\s*=\s*\{[^}]*?\brev\s*=\s*")(?<rev>[0-9a-f]{40})(")'
  $match = [regex]::Match($content, $pattern)
  if (-not $match.Success) {
    throw "Could not find a pinned 'godot' rev in $CargoTomlPath"
  }

  return @{
    Content = $content
    Match = $match
    Rev = $match.Groups['rev'].Value
  }
}

function Get-CurrentGodotRev([string]$CargoTomlPath) {
  $matchInfo = Get-GodotRevMatch -CargoTomlPath $CargoTomlPath
  return $matchInfo.Rev
}

function Write-Utf8NoBom([string]$Path, [string]$Content) {
  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Set-GodotRev([string]$CargoTomlPath, [string]$NewRev) {
  $matchInfo = Get-GodotRevMatch -CargoTomlPath $CargoTomlPath
  $content = $matchInfo.Content
  $revGroup = $matchInfo.Match.Groups['rev']

  if ($revGroup.Value -eq $NewRev) {
    return $false
  }

  $updated = $content.Substring(0, $revGroup.Index) + $NewRev + $content.Substring($revGroup.Index + $revGroup.Length)
  Write-Utf8NoBom -Path $CargoTomlPath -Content $updated
  return $true
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$rustDir = Join-Path $repoRoot "rust"
$cargoTomlPath = Join-Path $rustDir "Cargo.toml"
$lockfilePath = Join-Path $rustDir "Cargo.lock"

if (-not (Test-Path $cargoTomlPath)) {
  throw "Cargo.toml not found: $cargoTomlPath"
}

$currentRev = Get-CurrentGodotRev -CargoTomlPath $cargoTomlPath
$latestRev = Get-LatestGitRev -RepositoryUrl $RepoUrl -BranchName $Branch

Write-Host "Current godot rev: $currentRev"
Write-Host "Latest  godot rev: $latestRev"

if ($currentRev -eq $latestRev) {
  Write-Host "godot dependency is already up to date."
  exit 0
}

if ($DryRun) {
  Write-Host "Dry run: would update rust/Cargo.toml to rev $latestRev"
  if (-not $SkipLockfile) {
    Write-Host "Dry run: would run 'cargo update -p godot --precise $latestRev'"
  }
  exit 0
}

if (-not $SkipLockfile) {
  Assert-CommandExists "cargo"
}

$originalCargoToml = Get-Content -Raw -LiteralPath $cargoTomlPath
$hadLockfile = Test-Path $lockfilePath
$originalLockfileBytes = $null
if ($hadLockfile) {
  $originalLockfileBytes = [System.IO.File]::ReadAllBytes($lockfilePath)
}

try {
  $updated = Set-GodotRev -CargoTomlPath $cargoTomlPath -NewRev $latestRev
  if (-not $updated) {
    throw "Cargo.toml was not updated."
  }

  if (-not $SkipLockfile) {
    Push-Location $rustDir
    try {
      & cargo update -p godot --precise $latestRev
      if ((Test-Path variable:LASTEXITCODE) -and $LASTEXITCODE -ne 0) {
        throw "cargo update failed."
      }
    } finally {
      Pop-Location
    }
  }
}
catch {
  Write-Utf8NoBom -Path $cargoTomlPath -Content $originalCargoToml
  if ($hadLockfile -and $null -ne $originalLockfileBytes) {
    [System.IO.File]::WriteAllBytes($lockfilePath, $originalLockfileBytes)
  }
  throw
}

Write-Host "Updated godot dependency to rev $latestRev"

