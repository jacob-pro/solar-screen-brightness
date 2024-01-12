Unicode True
!addplugindir plugins
!addincludedir include

!include "MUI2.nsh"
!include "nsProcess.nsh"

RequestExecutionLevel user
OutFile "ssb-installer.exe"
Name "Solar Screen Brightness"
InstallDir "$LocalAppdata\solar-screen-brightness"

!define SHORT_CUT "$SMPROGRAMS\Solar Screen Brightness.lnk"
!define SHORT_CUT_UNINSTALL "$SMPROGRAMS\Uninstall Solar Screen Brightness.lnk"
!define SHORT_CUT_START_UP "$SMSTARTUP\Solar Screen Brightness (Minimised).lnk"

!define MUI_ICON "..\assets\icon-256.ico"
!define MUI_WELCOMEPAGE_TITLE "Solar Screen Brightness"
!define MUI_WELCOMEPAGE_TEXT "Click Install to start the installation."
!define MUI_FINISHPAGE_RUN "$INSTDIR\ssb.exe"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!define MUI_WELCOMEPAGE_TITLE "Solar Screen Brightness"
!define MUI_WELCOMEPAGE_TEXT "Click Uninstall to start the uninstallation."

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section
    SetOutPath "$INSTDIR"

    # Stop version 1
    ${nsProcess::KillProcess} "solar-screen-brightness.exe" $R0
    DetailPrint "Stopping solar-screen-brightness.exe code: $R0"
    # Stop version 2
    ${nsProcess::KillProcess} "ssb.exe" $R0
    DetailPrint "Stopping ssb.exe code: $R0"

    # Allow time for processes to stop
    Sleep 500

    # Delete version 1
    RMDir /r "$APPDATA\Solar Screen Brightness"

    File "..\target\release\ssb.exe"
    File "..\target\release\ssb-cli.exe"
    WriteUninstaller "$INSTDIR\uninstall.exe"
    CreateShortcut "${SHORT_CUT}" "$INSTDIR\ssb.exe"
    CreateShortcut "${SHORT_CUT_UNINSTALL}" "$INSTDIR\uninstall.exe"
    CreateShortcut "${SHORT_CUT_START_UP}" "$INSTDIR\ssb.exe" "--minimised"
SectionEnd

Section "uninstall"
    # Stop version 2
    ${nsProcess::KillProcess} "ssb.exe" $R0
    DetailPrint "Stopping ssb.exe code: $R0"

    # Allow time for processes to stop
    Sleep 500

    Delete "${SHORT_CUT}"
    Delete "${SHORT_CUT_UNINSTALL}"
    Delete "${SHORT_CUT_START_UP}"
    RMDir /r "$INSTDIR"
SectionEnd

