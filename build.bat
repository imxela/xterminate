@ECHO OFF

CALL cargo build --release

ROBOCOPY "%UserProfile%\Desktop\xterminate-master\res" "%UserProfile%\Desktop\xterminate-master\target\release\res" /E
ROBOCOPY "%UserProfile%\Desktop\xterminate-master" "%UserProfile%\Desktop\xterminate-master\target\release" "LICENSE"

PAUSE