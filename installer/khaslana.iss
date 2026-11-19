; Klado Windows 用户级安装脚本，沿用 Khaslana 的卸载身份。
;
; 用户级安装（免管理员）：默认装到 {localappdata}\Programs\Klado，
; 与应用内「移动到安全目录」的搬迁目标一致；运行期创建的 data\ 目录
; 卸载时不删除（Inno 只清理它自己安装的文件），数据不随卸载丢失。
;
; 版本号由 CI 经 /DAppVersion=x.y.z 注入；本地构建直接用 `cargo setup`
; 一键完成（构建 + 组包 + 编译，见 .cargo/config.toml 的 alias），或手动：
;   cargo build --release --bin klado --bin klado_updater
;   mkdir dist\package （拷贝两个 exe + LICENSE + README.md）
;   "C:\Program Files\Inno Setup 7\ISCC.exe" /DAppVersion=1.0.9 installer\khaslana.iss
; 产物输出到 dist\。

#ifndef AppVersion
; 未注入版本时给个可识别的占位，避免编译失败。
#define AppVersion "0.0.0-dev"
#endif

; 源文件目录：CI 与本地统一从仓库根的 dist/package 取（与便携 zip 同一
; 内容）。注意 Inno 的相对路径相对本脚本所在目录解析，需要 ..\ 回到仓库根。
#ifndef PayloadDir
#define PayloadDir "..\dist\package"
#endif

[Setup]
AppId={{5A1F3C92-7B4E-4D8A-9F60-3E2B8C7DA941}
AppName=Klado
AppVersion={#AppVersion}
AppPublisher=Klado
AppPublisherURL=https://cnb.cool/suhoan/khaslana
; 安装向导展示 Apache-2.0 许可证。
LicenseFile={#PayloadDir}\LICENSE
; 用户级安装：不需要管理员权限，默认目录与应用内搬迁目标一致。
PrivilegesRequired=lowest
DefaultDirName={code:DefaultKladoDirectory}
UsePreviousAppDir=no
DisableProgramGroupPage=yes
; 目录已存在（升级/搬迁过的用户）时直接沿用，不弹询问。
DirExistsWarning=no
; 相对本脚本回退到仓库根的 dist/ 输出。
OutputDir=..\dist
OutputBaseFilename=klado-setup-v{#AppVersion}-windows-x86_64
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; x64compatible：x64 与 ARM64（模拟运行）都允许安装；Inno 7 中旧的
; "x64" 标识已弃用（会替换为 x64os 并告警）。
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Uninstallable=yes
UninstallDisplayName=Klado
; 中文环境默认中文向导，其它语言回退英文。
ShowLanguageDialog=no

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
; 桌面图标默认不勾，避免捆绑软件观感。
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#PayloadDir}\klado.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\klado_updater.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#PayloadDir}\OPENSSL-LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\Klado"; Filename: "{app}\klado.exe"
Name: "{autodesktop}\Klado"; Filename: "{app}\klado.exe"; Tasks: desktopicon

[Run]
; 安装完成页提供「立即运行」勾选。
Filename: "{app}\klado.exe"; Parameters: "--complete-klado-migration"; Description: "{cm:LaunchProgram,Klado}"; Flags: nowait postinstall skipifsilent

[Code]
const
  UninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{5A1F3C92-7B4E-4D8A-9F60-3E2B8C7DA941}_is1';

var
  OldDirectory: String;
  OldUninstaller: String;

function InitializeSetup(): Boolean;
begin
  if RegQueryStringValue(HKCU, UninstallKey, 'Inno Setup: App Path', OldDirectory) then
    if FileExists(AddBackslash(OldDirectory) + 'khaslana.exe') and
       not FileExists(AddBackslash(OldDirectory) + 'klado.exe') then
      RegQueryStringValue(HKCU, UninstallKey, 'UninstallString', OldUninstaller)
    else
      OldUninstaller := '';
  Result := True;
end;

function DefaultKladoDirectory(Param: String): String;
begin
  if OldDirectory = '' then
    Result := ExpandConstant('{localappdata}\Programs\Klado')
  else if OldUninstaller <> '' then
    Result := AddBackslash(ExtractFileDir(OldDirectory)) + 'Klado'
  else
    Result := OldDirectory;
end;

function CalledByKladoUpdater(): Boolean;
var
  I: Integer;
begin
  Result := True;
  for I := 1 to ParamCount do
    if CompareText(ParamStr(I), '/KLADOUPGRADE') = 0 then
      Exit;
  Result := False;
end;

procedure DeleteLegacyShortcut(const Shortcut, LegacyExe: String);
var
  Shell: Variant;
  Link: Variant;
begin
  if not FileExists(Shortcut) then
    Exit;
  try
    Shell := CreateOleObject('WScript.Shell');
    Link := Shell.CreateShortcut(Shortcut);
    if CompareText(Link.TargetPath, LegacyExe) = 0 then
      DeleteFile(Shortcut);
  except
    Log('Could not verify the target of ' + Shortcut);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  TargetDirectory: String;
  LegacyExe: String;
  ResultCode: Integer;
begin
  if (CurStep <> ssPostInstall) or (OldUninstaller = '') then
    Exit;
  TargetDirectory := ExpandConstant('{app}');
  LegacyExe := AddBackslash(OldDirectory) + 'khaslana.exe';
  if not CalledByKladoUpdater() then
    if not Exec(AddBackslash(TargetDirectory) + 'klado_updater.exe',
      '--migrate-installed-data "' + LegacyExe + '" "' + AddBackslash(TargetDirectory) + 'klado.exe"',
      TargetDirectory, SW_HIDE, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
      Exit;

  DeleteLegacyShortcut(ExpandConstant('{autoprograms}\Khaslana.lnk'), LegacyExe);
  DeleteLegacyShortcut(ExpandConstant('{autodesktop}\Khaslana.lnk'), LegacyExe);
  OldUninstaller := RemoveQuotes(OldUninstaller);
  if (CompareText(OldDirectory, TargetDirectory) <> 0) and
     (CompareText(ExtractFileDir(OldUninstaller), OldDirectory) = 0) then
    RenameFile(OldUninstaller, OldUninstaller + '.pre-klado');
end;
