@echo off
REM THE CHECK'S REPLAY. Every fault of the week and its clean twin, judged by the organ.
REM A check that has never fired on a real violation is not a check. Organ must be up on
REM :8421 and the handshake built (cargo build --release).
REM
REM The books the fixtures are judged against ship WITH the fixtures (tests\judge\*.sarge),
REM so a stranger can run this from the clone. (The release review, 18 Sep: the first version pointed
REM at two repos on one machine, and could not be re-run by anyone else.)
cd /d "%~dp0"
set "H=target\release\handshake.exe"
set "D=tests\judge"
set "CS=%~dp0tests\judge\csharp.sarge"
set "JS=%~dp0tests\judge\node.sarge"
echo === C# files against tests\judge\csharp.sarge (+ book\universal.sarge) ===
for %%f in (cs-ensuredeleted-bad.cs cs-ensuredeleted-good.cs cs-newcontext-bad.cs cs-appservices-bad.cs cs-addcontrollers-bad.cs cs-clean-good.cs) do (
  echo --- %%f
  %H% --check "%D%" --file %%f --repo judge --rules "%CS%"
)
echo === JS files against tests\judge\node.sarge (+ book\universal.sarge) ===
for %%f in (js-hardcoded-db-bad.js js-await-sync-bad.js js-clean-good.js js-hardcoded-table-bad.jsx) do (
  echo --- %%f
  %H% --check "%D%" --file %%f --repo judge --rules "%JS%"
)
