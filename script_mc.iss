[Setup]
AppName=Paraguacraft
AppVersion=3.0.0
AppPublisher=Jaful
DefaultDirName={userappdata}\Paraguacraft
DefaultGroupName=Paraguacraft
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\Paraguacraft.exe
SetupIconFile=C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft\iconomc.ico
Compression=lzma2/ultra
SolidCompression=yes
OutputDir=C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft
OutputBaseFilename=Instalar_Paraguacraft_v3.0.0

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft\dist\Paraguacraft.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"
; Cambiamos autodesktop por userdesktop para que no pida permisos al crear el acceso directo
Name: "{userdesktop}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Paraguacraft.exe"; Description: "{cm:LaunchProgram,Paraguacraft}"; Flags: nowait postinstall skipifsilent