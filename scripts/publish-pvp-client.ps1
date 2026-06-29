# Sube ParaguacraftPvP + OptiFine a GitHub Releases (descarga global del launcher).
# Requisito: gh auth login
param(
    [string]$Tag = "pvp-client-2.1.2"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

function Resolve-GhExe {
    $cmd = Get-Command gh -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $candidates = @(
        "$env:ProgramFiles\GitHub CLI\gh.exe",
        "${env:ProgramFiles(x86)}\GitHub CLI\gh.exe"
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) { return $path }
    }
    throw "GitHub CLI no encontrado. Instalalo desde https://cli.github.com/"
}

$gh = Resolve-GhExe
Write-Host "Usando: $gh"

$clientJar = Join-Path $Root "bundled\pvp\ParaguacraftPvP-2.1.2.jar"
$optifineJar = Join-Path $Root "bundled\pvp\OptiFine_1.8.9_HD_U_M5.jar"
if (-not (Test-Path $clientJar)) { throw "Falta: $clientJar" }
if (-not (Test-Path $optifineJar)) { throw "Falta: $optifineJar" }

$sha1 = (Get-FileHash $clientJar -Algorithm SHA1).Hash.ToLower()
Write-Host "ParaguacraftPvP SHA1: $sha1"

& $gh release create $Tag `
    --repo SantiJ10/Paraguacraft `
    --title "Paraguacraft PvP Client 2.1.2" `
    --notes "Cliente PvP Forge 1.8.9 + OptiFine. SHA1 mod: $sha1" `
    $clientJar `
    $optifineJar

Write-Host "Listo: https://github.com/SantiJ10/Paraguacraft/releases/tag/$Tag"
