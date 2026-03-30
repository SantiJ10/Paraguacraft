[Setup]
AppName=Paraguacraft
AppVersion=1.0.2
AppPublisher=Jaful
DefaultDirName={autopf}\Paraguacraft
DefaultGroupName=Paraguacraft
UninstallDisplayIcon={app}\Paraguacraft.exe
SetupIconFile=C:\Users\San\Documents\Amin\Proyectos\paraguacraft\iconomc.ico
Compression=lzma2/ultra
SolidCompression=yes
OutputDir=C:\Users\San\Documents\Amin\Proyectos\paraguacraft
OutputBaseFilename=Instalar_Paraguacraft

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "C:\Users\San\Documents\Amin\Proyectos\paraguacraft\dist\Paraguacraft.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"
Name: "{autodesktop}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Paraguacraft.exe"; Description: "{cm:LaunchProgram,Paraguacraft}"; Flags: nowait postinstall skipifsilent