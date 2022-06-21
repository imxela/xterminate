@ECHO OFF

CALL cargo build --release

ROBOCOPY "%UserProfile%\Desktop\xterminate-main\res" "%UserProfile%\Desktop\xterminate-main\target\release\res" /E
ROBOCOPY "%UserProfile%\Desktop\xterminate-main" "%UserProfile%\Desktop\xterminate-main\target\release" "LICENSE"

PAUSE