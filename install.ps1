$ErrorActionPreference = "Stop"

$Repo = "movecx/claude-box"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\claude-box" }

function Get-LatestVersion {
    $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $response.tag_name
}

function Main {
    $Platform = "x86_64-pc-windows-msvc"
    Write-Host "Platform: $Platform"

    $Version = if ($env:VERSION) { $env:VERSION } else { Get-LatestVersion }
    if (-not $Version) {
        Write-Error "Failed to get latest version"
        exit 1
    }
    Write-Host "Installing claude-box $Version"

    $DownloadUrl = "https://github.com/$Repo/releases/download/$Version/claude-box-$Platform.zip"
    $TempZip = "$env:TEMP\claude-box.zip"

    # Create install directory
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

    # Download
    Write-Host "Downloading from $DownloadUrl"
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

    # Extract
    Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
    Remove-Item $TempZip

    Write-Host "Installed claude-box to $InstallDir\claude-box.exe"

    # Check if install dir is in PATH
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        $AddToPath = Read-Host "Add $InstallDir to PATH? (y/n)"
        if ($AddToPath -eq "y") {
            [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
            Write-Host "Added to PATH. Restart your terminal for changes to take effect."
        } else {
            Write-Host ""
            Write-Host "To add manually, run:"
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$InstallDir', 'User')"
        }
    }
}

Main
