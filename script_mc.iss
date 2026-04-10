[Setup]
AppName=Paraguacraft
AppVersion=4.2.0
AppPublisher=Jaful
DefaultDirName={userappdata}\Paraguacraft
DefaultGroupName=Paraguacraft
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\Paraguacraft.exe
SetupIconFile=C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft\iconomc.ico
Compression=lzma2/ultra
SolidCompression=yes
OutputDir=C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft
OutputBaseFilename=Instalar_Paraguacraft_v4.2.0

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft\dist\Paraguacraft.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"
Name: "{userdesktop}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Paraguacraft.exe"; Description: "{cm:LaunchProgram,Paraguacraft}"; Flags: nowait postinstall skipifsilent