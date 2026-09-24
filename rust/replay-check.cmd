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
REM The C# half was archived on 22 Sep - the Captain's ruling that Sarge releases for Node
REM only. The fixtures and csharp.sarge are whole in docs\archive\csharp-fixtures\ with the
REM reason; restoring them restores this block. Nothing else referenced them.
set "JS=%~dp0tests\judge\node.sarge"
echo === JS files against tests\judge\node.sarge (+ book\universal.sarge) ===
for %%f in (js-hardcoded-db-bad.js js-await-sync-bad.js js-clean-good.js js-hardcoded-table-bad.jsx) do (
  echo --- %%f
  %H% --check "%D%" --file %%f --repo judge --rules "%JS%"
)
REM THE LONG FILES, added 24 Sep, judged against the SHIPPED book rather than the fixtures'
REM own copy. Two bugs lived a week in the gap between a 16-line fixture and a real file:
REM a long faulty file reported CHECKS PASSED, and 69 lines of correct code drew twelve
REM false flags. Short fixtures measure a check on short fixtures.
echo.
echo === the long files against the SHIPPED book\node.sarge ===
for %%f in (js-long-bad.js js-long-good.js js-nosarge-bad.js js-clean2-good.js js-clean3-good.js js-stream-bad.js) do (
  echo --- %%f
  %H% --check "%D%" --file %%f --repo judge --rules "%~dp0..\book\node.sarge"
)
