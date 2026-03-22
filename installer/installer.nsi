; MediaChat Native — NSIS installer
; Usage: makensis /DVERSION=x.y.z installer/installer.nsi

!ifndef VERSION
  !error "Pass /DVERSION=x.y.z to makensis"
!endif

!define APP_NAME     "MediaChat"
!define APP_EXE      "mediachat-native.exe"
!define INSTALL_DIR  "$LOCALAPPDATA\MediaChat"
!define REG_RUN      "Software\Microsoft\Windows\CurrentVersion\Run"
!define REG_UNINST   "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"

Name            "${APP_NAME}"
OutFile         "MediaChat-Setup-${VERSION}.exe"
InstallDir      "${INSTALL_DIR}"
RequestExecutionLevel user   ; LOCALAPPDATA — no admin required

Icon "..\installerIcon.ico"
UninstallIcon "..\installerIcon.ico"

; ── Pages ────────────────────────────────────────────────────────────────────
Page instfiles
UninstPage instfiles

; ── Install ──────────────────────────────────────────────────────────────────
Section "Install"
  SetOutPath "$INSTDIR"
  File "..\target\release\${APP_EXE}"

  ; Launch at Windows startup (current user)
  WriteRegStr HKCU "${REG_RUN}" "${APP_NAME}" '"$INSTDIR\${APP_EXE}"'

  ; Add/Remove Programs entry
  WriteRegStr   HKCU "${REG_UNINST}" "DisplayName"          "${APP_NAME}"
  WriteRegStr   HKCU "${REG_UNINST}" "DisplayVersion"       "${VERSION}"
  WriteRegStr   HKCU "${REG_UNINST}" "Publisher"            "ProbablyClem"
  WriteRegStr   HKCU "${REG_UNINST}" "UninstallString"      '"$INSTDIR\Uninstall.exe"'
  WriteRegStr   HKCU "${REG_UNINST}" "DisplayIcon"          '"$INSTDIR\${APP_EXE}"'
  WriteRegStr   HKCU "${REG_UNINST}" "InstallLocation"      "$INSTDIR"
  WriteRegDWORD HKCU "${REG_UNINST}" "NoModify"             1
  WriteRegDWORD HKCU "${REG_UNINST}" "NoRepair"             1
  WriteRegDWORD HKCU "${REG_UNINST}" "SystemComponent"      0

  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Launch the app after installation
  Exec '"$INSTDIR\${APP_EXE}"'
SectionEnd

; ── Uninstall ─────────────────────────────────────────────────────────────────
Section "Uninstall"
  ; Kill the process if running
  ExecWait 'taskkill /F /IM "${APP_EXE}"'

  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir  "$INSTDIR"

  DeleteRegValue HKCU "${REG_RUN}"   "${APP_NAME}"
  DeleteRegKey   HKCU "${REG_UNINST}"
SectionEnd
