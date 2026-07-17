# Sube los texture packs PvP 1.21.11 a GitHub Releases.
param(
    [string]$Tag = "pvp-packs-modern-1.0",
    [string]$PacksDir = "$PSScriptRoot\..\bundled\pvp-modern\resourcepacks"
)

$ErrorActionPreference = "Stop"

function Resolve-GhExe {
    $cmd = Get-Command gh -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $candidates = @(
        "$env:ProgramFiles\GitHub CLI\gh.exe",
        "${env:ProgramFiles(x86)}\GitHub CLI\gh.exe",
        "$env:LOCALAPPDATA\Programs\GitHub CLI\gh.exe"
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) { return $path }
    }
    throw "GitHub CLI no encontrado."
}

$gh = Resolve-GhExe
$files = Get-ChildItem -Path $PacksDir -Filter "*.zip"
if ($files.Count -eq 0) { throw "No hay .zip en $PacksDir" }

& $gh release create $Tag `
    --repo SantiJ10/Paraguacraft `
    --title "Paraguacraft PvP Texture Packs 1.21.11" `
    --notes "Resource packs PvP 1.21.11. Catalogo: clientes/paraguacraft-pvp-modern/packs/catalog.json" `
    @($files | ForEach-Object { $_.FullName })

Write-Host "Release: https://github.com/SantiJ10/Paraguacraft/releases/tag/$Tag"
