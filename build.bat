@ECHO OFF

CALL cargo build --release

ROBOCOPY .\res .\target\release\res /E
ROBOCOPY . .\target\release LICENSE

PAUSE