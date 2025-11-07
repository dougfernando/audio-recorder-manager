@echo off
REM Temporarily modify PATH to exclude Git's link.exe
set "OLDPATH=%PATH%"
set "PATH=%PATH:C:\Program Files\Git\usr\bin;=%"
set "PATH=%PATH:;C:\Program Files\Git\usr\bin=%"

REM Build the project
cargo build --release

REM Restore PATH
set "PATH=%OLDPATH%"
