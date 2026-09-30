# Installs the raytha CLI on Windows (x64).
#   irm <url-of-this-script> | iex
# Environment: RAYTHA_VERSION (default latest), RAYTHA_INSTALL_DIR (default %LOCALAPPDATA%\raytha\bin),
# RAYTHA_INSTALL_BASE (release host; same layout as install.sh).
$ErrorActionPreference = 'Stop'

$Base = if ($env:RAYTHA_INSTALL_BASE) { $env:RAYTHA_INSTALL_BASE } else { 'https://github.com/RaythaHQ/raytha-cli/releases' }
$Version = if ($env:RAYTHA_VERSION) { $env:RAYTHA_VERSION } else { 'latest' }
$InstallDir = if ($env:RAYTHA_INSTALL_DIR) { $env:RAYTHA_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'raytha\bin' }

if (-not [Environment]::Is64BitOperatingSystem) { throw 'Only 64-bit Windows is supported.' }
$Asset = 'raytha-x86_64-pc-windows-msvc.zip'
$Root = if ($Version -eq 'latest') { "$Base/latest/download" } else { "$Base/download/$Version" }

$Tmp = Join-Path ([IO.Path]::GetTempPath()) ("raytha-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Tmp | Out-Null
try {
  Write-Host "Downloading $Asset ($Version)"
  Invoke-WebRequest -UseBasicParsing "$Root/$Asset" -OutFile (Join-Path $Tmp $Asset)
  Invoke-WebRequest -UseBasicParsing "$Root/SHA256SUMS" -OutFile (Join-Path $Tmp 'SHA256SUMS')

  $line = Get-Content (Join-Path $Tmp 'SHA256SUMS') | Where-Object { $_ -match "  $([regex]::Escape($Asset))$" } | Select-Object -First 1
  if (-not $line) { throw "$Asset is not listed in SHA256SUMS" }
  $expected = ($line -split '\s+')[0].ToLower()
  $actual = (Get-FileHash (Join-Path $Tmp $Asset) -Algorithm SHA256).Hash.ToLower()
  if ($expected -ne $actual) { throw "Checksum mismatch for $Asset (expected $expected, got $actual)" }

  Expand-Archive -Path (Join-Path $Tmp $Asset) -DestinationPath $Tmp -Force
  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
  Copy-Item (Join-Path $Tmp 'raytha.exe') (Join-Path $InstallDir 'raytha.exe') -Force
  Write-Host "Installed to $InstallDir\raytha.exe"

  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  if (($userPath -split ';') -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable('Path', "$userPath;$InstallDir", 'User')
    Write-Host "Added $InstallDir to your user PATH. Open a new terminal."
  }
  Write-Host "Next: set RAYTHA_URL and RAYTHA_API_KEY, then run 'raytha doctor' and 'raytha guide'."
}
finally {
  Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}
