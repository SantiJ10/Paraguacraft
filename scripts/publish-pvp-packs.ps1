# Sube los texture packs PvP a GitHub Releases (descarga global para todos los usuarios).
# Requisito: gh auth login
param(
    [string]$Tag = "pvp-packs-1.0",
    [string]$PacksDir = "$PSScriptRoot\..\bundled\pvp\resourcepacks"
)

$ErrorActionPreference = "Stop"

function Resolve-GhExe {
    $cmd = Get-Command gh -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }
    $candidates = @(
        "$env:ProgramFiles\GitHub CLI\gh.exe",
        "${env:ProgramFiles(x86)}\GitHub CLI\gh.exe",
        "$env:LOCALAPPDATA\Programs\GitHub CLI\gh.exe"
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) {
            return $path
        }
    }
    throw "GitHub CLI no encontrado. Instalalo desde https://cli.github.com/ y reinicia PowerShell."
}

$gh = Resolve-GhExe
Write-Host "Usando: $gh"

$files = Get-ChildItem -Path $PacksDir -Filter "*.zip"
if ($files.Count -eq 0) {
    throw "No hay .zip en $PacksDir"
}

Write-Host "Creando release $Tag con $($files.Count) packs..."
$assetArgs = @()
foreach ($f in $files) {
    $assetArgs += $f.FullName
}

& $gh release create $Tag `
    --repo SantiJ10/Paraguacraft `
    --title "Paraguacraft PvP Texture Packs" `
    --notes "Resource packs PvP 1.8.9 para el cliente Paraguacraft. Catalogo en clientes/paraguacraft-pvp/packs/catalog.json" `
    @assetArgs

Write-Host "Listo. URLs: https://github.com/SantiJ10/Paraguacraft/releases/download/$Tag/<archivo>.zip"
