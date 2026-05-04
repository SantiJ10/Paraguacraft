[Setup]
AppName=Paraguacraft
AppVersion=5.4.0
AppPublisher=Jaful
DefaultDirName={autopf}\Paraguacraft
DefaultGroupName=Paraguacraft
PrivilegesRequired=admin
UninstallDisplayIcon={app}\Paraguacraft.exe
SetupIconFile=C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft\iconomc.ico
Compression=lzma2/ultra
SolidCompression=yes
OutputDir=C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft
OutputBaseFilename=Instalar_Paraguacraft_v5.4.0

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "C:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft\dist\Paraguacraft.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"; WorkingDir: "{userappdata}"
Name: "{commondesktop}\Paraguacraft"; Filename: "{app}\Paraguacraft.exe"; WorkingDir: "{userappdata}"; Tasks: desktopicon

[Run]
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; \
  Description: "Instalando Microsoft WebView2 Runtime..."; \
  Flags: waituntilterminated runhidden; Check: not IsWebView2Installed
Filename: "{app}\Paraguacraft.exe"; Description: "{cm:LaunchProgram,Paraguacraft}"; Flags: nowait postinstall skipifsilent

[Code]
function IsWebView2Installed: Boolean;
var
  Ver: String;
begin
  Result := False;
  if RegQueryStringValue(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Ver) then
    Result := (Ver <> '') and (Ver <> '0.0.0.0')
  else if RegQueryStringValue(HKLM, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Ver) then
    Result := (Ver <> '') and (Ver <> '0.0.0.0')
  else if RegQueryStringValue(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Ver) then
    Result := (Ver <> '') and (Ver <> '0.0.0.0');
end;

procedure InitializeWizard;
begin
  if not IsWebView2Installed then
    MsgBox('Paraguacraft necesita Microsoft WebView2 Runtime.' + #13#10 +
           'Se va a descargar e instalar automáticamente durante la instalación.' + #13#10 +
           'Asegurate de tener conexión a internet.', mbInformation, MB_OK);
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  TmpFile, PSCmd: String;
  ResultCode: Integer;
begin
  if (CurStep = ssInstall) and not IsWebView2Installed then begin
    TmpFile := ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe');
    if not FileExists(TmpFile) then begin
      PSCmd := '-NonInteractive -NoProfile -ExecutionPolicy Bypass -Command ' +
               '"Invoke-WebRequest -Uri ''https://go.microsoft.com/fwlink/p/?LinkId=2124703'' ' +
               '-OutFile ''' + TmpFile + '''"';
      Exec('powershell.exe', PSCmd, '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    end;
  end;
end;