# Empaqueta Paraguacraft PvP Client 2.0.0 + plugin de badges.
# Uso: .\scripts\release-pvp.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Write-Host "==> Build cliente Forge..."
Push-Location "$Root\client"
& .\gradlew.bat build --quiet
if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
Pop-Location

Write-Host "==> Build plugin Paper..."
Push-Location "$Root\server\paraguacraft-badges"
if (Test-Path "$Root\client\gradlew.bat") {
    & "$Root\client\gradlew.bat" -p "$Root\server\paraguacraft-badges" build --quiet
} else {
    gradle build --quiet
}
if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
Pop-Location

$ClientJar = Get-ChildItem "$Root\client\build\libs\ParaguacraftPvP-*.jar" | Where-Object { $_.Name -notmatch "sources|dev" } | Sort-Object Name -Descending | Select-Object -First 1
$PluginJar = Get-ChildItem "$Root\server\paraguacraft-badges\build\libs\ParaguacraftBadges-*.jar" | Select-Object -First 1

if (-not $ClientJar) { throw "No se encontro JAR del cliente" }
if (-not $PluginJar) { throw "No se encontro JAR del plugin" }

$BundledPvp = "$Root\bundled\pvp"
$ClientesPvp = "$Root\clientes\paraguacraft-pvp"
$BundledServer = "$Root\bundled\server"
New-Item -ItemType Directory -Force -Path $BundledPvp, $ClientesPvp, $BundledServer | Out-Null

$DestClient = Join-Path $BundledPvp $ClientJar.Name
$DestClientes = Join-Path $ClientesPvp $ClientJar.Name
$DestPlugin = Join-Path $BundledServer "ParaguacraftBadges-1.0.0.jar"
Copy-Item $ClientJar.FullName $DestClient -Force
Copy-Item $ClientJar.FullName $DestClientes -Force
Copy-Item $PluginJar.FullName $DestPlugin -Force

$ClientSha1 = (Get-FileHash $DestClient -Algorithm SHA1).Hash.ToLower()
$PluginSha1 = (Get-FileHash $DestPlugin -Algorithm SHA1).Hash.ToLower()

Write-Host ""
Write-Host "Cliente: $DestClient"
Write-Host "  SHA1: $ClientSha1"
Write-Host "Plugin:  $DestPlugin"
Write-Host "  SHA1: $PluginSha1"
Write-Host ""
Write-Host "Actualiza launcher/src-tauri/src/core/loaders/pvp.rs con:"
Write-Host "  PVP_CLIENT_JAR = `"$($ClientJar.Name)`""
Write-Host "  PVP_CLIENT_SHA1 = `"$ClientSha1`""
Write-Host "  RELEASE_TAG = `"pvp-client-2.0.0`""
Write-Host ""
Write-Host "Subi clientes/paraguacraft-pvp/ y bundled/server/ al repo (git push)."

# Actualizar manifest del cliente
$ManifestClient = "$Root\client\src\main\java\com\paraguacraft\pvp\manifest.json"
$ManifestClientes = "$Root\clientes\paraguacraft-pvp\manifest.json"
$json = @"
{
  "id": "paraguacraft-pvp",
  "nombre": "Paraguacraft PvP",
  "mc_version": "1.8.9",
  "loader": "forge",
  "descripcion": "Cliente PvP 2.0.0 — HUD/GUI Lunar + resource packs + badge sync.",
  "mods": [
    {
      "filename": "$($ClientJar.Name)",
      "sha1": "$ClientSha1"
    },
    {
      "filename": "OptiFine_1.8.9_HD_U_M5.jar",
      "sha1": "d362d58a28f5373b141b9e426e8e160638bfafcd"
    }
  ]
}
"@
Set-Content -Path $ManifestClient -Value $json -Encoding UTF8
if (Test-Path $ManifestClientes) {
    Set-Content -Path $ManifestClientes -Value $json -Encoding UTF8
}

Write-Host "Manifests actualizados."
