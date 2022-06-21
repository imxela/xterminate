@ECHO OFF

CALL cargo build --release

ROBOCOPY "%UserProfile%\Desktop\xterminate-master\res" "%UserProfile%\Desktop\xterminate-master\target\release\res" /E

PAUSE