# Empaqueta el cliente ParaguacraftPvP en la carpeta del repo (bundled/pvp).
# NOTA: el cliente PvP NO se publica en GitHub Releases. El launcher lo toma
# desde bundled/pvp (embebido en el instalador y/o vía raw.githubusercontent).
# Solo el instalador del launcher se sube como release (v7.x.x).

param(
    [string]$Version = "2.1.3"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

$builtJar = Join-Path $Root "client\build\libs\ParaguacraftPvP-$Version.jar"
$bundledDir = Join-Path $Root "bundled\pvp"
$resourcesDir = Join-Path $Root "launcher\src-tauri\resources\bundled\pvp"
$targetJar = Join-Path $bundledDir "ParaguacraftPvP-$Version.jar"

if (-not (Test-Path $builtJar)) { throw "Falta el JAR compilado: $builtJar (corré gradlew build)" }

New-Item -ItemType Directory -Force -Path $bundledDir | Out-Null
New-Item -ItemType Directory -Force -Path $resourcesDir | Out-Null

Copy-Item $builtJar $targetJar -Force
Copy-Item $builtJar (Join-Path $resourcesDir "ParaguacraftPvP-$Version.jar") -Force

# Limpia versiones viejas del cliente en ambas carpetas.
Get-ChildItem $bundledDir -Filter "ParaguacraftPvP-*.jar" | Where-Object { $_.Name -ne "ParaguacraftPvP-$Version.jar" } | Remove-Item -Force -ErrorAction SilentlyContinue
Get-ChildItem $resourcesDir -Filter "ParaguacraftPvP-*.jar" | Where-Object { $_.Name -ne "ParaguacraftPvP-$Version.jar" } | Remove-Item -Force -ErrorAction SilentlyContinue

$sha1 = (Get-FileHash $targetJar -Algorithm SHA1).Hash.ToLower()
Write-Host "ParaguacraftPvP-$Version.jar SHA1: $sha1"
Write-Host "Copiado a:"
Write-Host "  $targetJar"
Write-Host "  $(Join-Path $resourcesDir ""ParaguacraftPvP-$Version.jar"")"
Write-Host "Actualizá el SHA1 en clientes/paraguacraft-pvp/manifest.json y en pvp.rs (FALLBACK_MODS)."
