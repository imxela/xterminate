Unicode true

!include "MUI2.nsh"
 
InstallDir $PROGRAMFILES64\xterminate

# retreive xterminate version and add it to the installer's name
!getdllversion "target\release\xterminate.exe" EXEVERSION
Name "xterminate v${EXEVERSION1}.${EXEVERSION2}.${EXEVERSION3}"

# set installer executable name
OutFile "target\release\xterminate-setup.exe"

RequestExecutionLevel admin

!define MUI_WELCOMEPAGE_TEXT "This setup will guide you through the installation of xterminate v${EXEVERSION1}.${EXEVERSION2}.${EXEVERSION3}$\r$\n$\r$\nIf you already have an older version of xterminate installed it is recommended that you uninstall it before continuing this setup.$\r$\n$\r$\nClick Next to continue."
!define MUI_LICENSEPAGE_CHECKBOX

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE LICENSE
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH


# run xterminate when setup is complete
Function .oninstsuccess   
Exec "$INSTDIR\xterminate.exe"   
FunctionEnd

!insertmacro MUI_UNPAGE_COMPONENTS
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"


Section

# used to set $APPDATA to be system-wide (i.e. %ProgramData%)
SetShellVarContext all
 
# define output path
SetOutPath $INSTDIR

# add files to uninstaller
File LICENSE
File target\release\xterminate.exe

SetOutPath $INSTDIR\res
File /r res\*.*
SetOutPath $INSTDIR
 
# create uninstaller
WriteUninstaller $INSTDIR\uninstall.exe

CreateShortcut "$SMPROGRAMS\xterminate.lnk" "$INSTDIR\xterminate.exe"
CreateShortcut "$SMPROGRAMS\Uninstall xterminate.lnk" "$INSTDIR\uninstall.exe"

# run xterminate on startup
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "xterminate" "$INSTDIR\xterminate.exe"

# add uninstaller to list of installed programs
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "DisplayName" "xterminate"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "UninstallString" "$INSTDIR\uninstall.exe"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "Publisher" "Xela"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "DisplayVersion" "${EXEVERSION1}.${EXEVERSION2}.${EXEVERSION3}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "VersionMajor" "${EXEVERSION1}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "VersionMinor" "${EXEVERSION2}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "Version" "${EXEVERSION3}"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "InstallLocation" "$INSTDIR"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                 "URLInfoAbout" "https://github.com/imxela/xterminate"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "HelpLink" "https://github.com/imxela/xterminate"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "Readme" "$INSTDIR\README"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "DisplayIcon" "$INSTDIR\xterminate.exe"
WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "NoModify" 1
WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate" \
                   "NoRepair" 1

SectionEnd



Section "Uninstall"
 
# used to set $APPDATA to be system-wide (i.e. %ProgramData%)
SetShellVarContext all

# remove xterminate program files
Delete "$INSTDIR\xterminate.exe"
Delete "$INSTDIR\LICENSE"
Delete "$INSTDIR\uninstall.exe"
Delete "$INSTDIR\res\icon.ico"
Delete "$INSTDIR\res\cursor.cur"
Delete "$INSTDIR\res\config.toml"
RMDir "$INSTDIR\res"
RMDir "$INSTDIR\" # non-destructive - removes only if empty

# remove logs
RMDir /r "$APPDATA\xterminate\logs\"

# remove program data
Delete "$APPDATA\xterminate\config.toml"
RMDir "$APPDATA\xterminate"

# remove xterminate shortcuts
Delete $SMPROGRAMS\xterminate.lnk
Delete "$SMPROGRAMS\Uninstall xterminate.lnk"

# remove run on startup registry value
DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "xterminate"

# remove from list of installed programs
DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\xterminate"


SectionEnd