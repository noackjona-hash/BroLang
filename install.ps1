# BroLang Compiler Global Installer Script for Windows

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $HOME ".brolang\bin"
Write-Host "Setting up BroLang CLI installation at: $InstallDir"

# 1. Create installation directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# 2. Compile BroLang in release mode
Write-Host "Building BroLang compiler in release mode..."
cargo build --release

$CompiledBin = Join-Path (Get-Location) "target\release\bro.exe"
if (-not (Test-Path $CompiledBin)) {
    Write-Error "Could not find compiled binary at $CompiledBin."
}

# 3. Copy compiler binary
Copy-Item -Path $CompiledBin -Destination $InstallDir -Force
Write-Host "Installed bro.exe successfully."

# 4. Download and extract FASM for Windows
Write-Host "Downloading FASM assembler dependencies..."
$ZipUrl = "https://flatassembler.net/fasmw17335.zip"
$ZipFile = Join-Path $env:TEMP "fasmw_temp.zip"
$ExtractDir = Join-Path $env:TEMP "fasmw_extracted"

# Clean up any leftover extraction directories
if (Test-Path $ExtractDir) {
    Remove-Item -Path $ExtractDir -Recurse -Force
}

# Download Zip using WebClient or Invoke-WebRequest
Invoke-WebRequest -Uri $ZipUrl -OutFile $ZipFile

# Extract Zip
Expand-Archive -Path $ZipFile -DestinationPath $ExtractDir

# Find and copy fasm.exe as fasm2.exe
$FasmSource = Get-ChildItem -Path $ExtractDir -Filter "fasm.exe" -Recurse | Select-Object -First 1
if ($FasmSource) {
    Copy-Item -Path $FasmSource.FullName -Destination (Join-Path $InstallDir "fasm2.exe") -Force
    Write-Host "Successfully installed fasm2.exe helper dependency."
} else {
    Write-Error "Could not locate fasm.exe inside downloaded package."
}

# Clean up temp files
Remove-Item -Path $ZipFile -Force
Remove-Item -Path $ExtractDir -Recurse -Force

# 5. Add installation directory to User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = $UserPath
    if ($NewPath -and -not $NewPath.EndsWith(";")) {
        $NewPath += ";"
    }
    $NewPath += $InstallDir
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    
    # Broadcast Path change to current process
    $env:Path += ";$InstallDir"
    Write-Host "Added $InstallDir to User PATH."
} else {
    Write-Host "$InstallDir is already in User PATH."
}

Write-Host "BroLang installation complete! Restart your shell and run 'bro' from anywhere."
