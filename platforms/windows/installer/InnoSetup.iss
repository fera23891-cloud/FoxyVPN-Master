[Setup]
AppName=FoxyVPN
AppVersion=2.0.0
DefaultDirName={autopf}\FoxyVPN
DefaultGroupName=FoxyVPN
OutputDir=..\..\..\dist
OutputBaseFilename=FoxyVPN-Setup-v2.0.0
Compression=lzma2/ultra64
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
DisableProgramGroupPage=yes

[Files]
Source: "..\..\..\target\release\foxyvpn-portable.exe"; DestDir: "{app}"; DestName: "FoxyVPN.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\FoxyVPN"; Filename: "{app}\FoxyVPN.exe"
Name: "{autodesktop}\FoxyVPN"; Filename: "{app}\FoxyVPN.exe"

[Run]
Filename: "{app}\FoxyVPN.exe"; Description: "Run FoxyVPN"; Flags: nowait postinstall skipifsilent
