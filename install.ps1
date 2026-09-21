# Sarge - one command.
#
#   irm https://raw.githubusercontent.com/1picassoai/Sarge/main/install.ps1 | iex
#
# What it does, in order, and nothing else:
#   1. checks you have Python 3.10+ and enough disk
#   2. clones the v0.1.0 tag into .\Sarge  (or uses the clone you are standing in)
#   3. downloads the prebuilt organ from the GitHub release          ~46 MB
#   4. downloads the CUDA runtime ONLY if you have an NVIDIA card   ~405 MB
#   5. downloads the model from Hugging Face                        ~2.4 GB
#   6. pip installs the Python harness into the clone
#   7. writes start-organ.cmd and start-console.cmd with YOUR paths
#   8. starts the organ and proves the check can catch a known-bad file
#
# Nothing is installed system-wide. Nothing leaves your machine. It asks nothing and
# guesses nothing. Every file it downloads is checked against a published sha256, and a
# mismatch deletes the file and stops the install.
#
# That last sentence was FALSE until 21 Sep - it claimed verification this script did not
# do. @Galahad's release review found it. If you change how a download works here, change
# this comment in the same commit or it becomes a lie again.

$ErrorActionPreference = "Stop"
$ProgressPreference    = "SilentlyContinue"   # much faster Invoke-WebRequest

# The tag the installer fetches. It pins the TREE, the ORGAN ASSET and the CUDA runtime,
# so all three must come from the same release or a stranger gets a mismatched pair.
# (the release review, 20 Sep: this said v0.1.0 while the script came from main, so the
# installer was new and the check it installed was twenty commits old.)
$TAG       = "v0.2.0"
$REPO      = "https://github.com/1picassoai/Sarge"
$MODEL     = "Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
$MODEL_URL = "https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/$MODEL"
$NEED_GB   = 4

# EVERY DOWNLOAD IS CHECKED. Added 21 Sep after @Galahad's release review: neither
# installer verified a single byte it fetched, while the comment at the top of this file
# claimed it "stops loudly on anything it cannot verify". It did not. It does now.
#
# The organ and cudart hashes are the assets signed into the v0.1.1 stamp and carried
# byte-identical into v0.2.0. The model hash is Hugging Face's own X-Linked-ETag,
# verified 21 Sep against the 2,497,281,120-byte copy on the Captain's disk.
$ORGAN_SHA256  = "3d336f87c3be11d3e217c9091922656f943e952188206bf7fba6eb94f3c0c92d"
$CUDART_SHA256 = "37d27a8ff3366f3f2d264dd693a88b0884f457985f00c9d0a416e74266155012"
$MODEL_SHA256  = "3605803b982cb64aead44f6c1b2ae36e3acdb41d8e46c8a94c6533bc4c67e597"

# A mismatch is not a warning. The file is deleted and the install stops - a half-trusted
# binary left on disk is worse than none, because the next run would find it already
# there and skip the download.
function Verify-Sha256 ($path, $want, $what) {
    $got = (Get-FileHash -Path $path -Algorithm SHA256).Hash.ToLower()
    if ($got -ne $want.ToLower()) {
        Remove-Item $path -Force -ErrorAction SilentlyContinue
        Stop-With "$what does not match what we published, and has been deleted.`n  expected  $want`n  got       $got`n  Do not run anything that was downloaded. Tell us: $REPO/discussions"
    }
    Good "$what verified (sha256 $($want.Substring(0,16))...)"
}

function Say      ($m) { Write-Host "  $m" }
function Step     ($m) { Write-Host "`n== $m" -ForegroundColor Cyan }
function Good     ($m) { Write-Host "  OK  $m" -ForegroundColor Green }
function Warn     ($m) { Write-Host "  !   $m" -ForegroundColor Yellow }
function Stop-With($m) { Write-Host "`nSTOPPED: $m`n" -ForegroundColor Red; exit 1 }

# A live bar, because a 2.4 GB download with no feedback looks like a hang.
function Show-Bar ($done, $total, $label) {
    $w = 34
    if ($total -gt 0) {
        $pct  = [math]::Min(100, [int](($done / $total) * 100))
        $fill = [int](($pct / 100) * $w)
        $bar  = ("#" * $fill).PadRight($w, ".")
        $txt  = "{0,-22} [{1}] {2,3}%  {3,6:N0} / {4:N0} MB" -f $label, $bar, $pct, ($done/1MB), ($total/1MB)
    } else {
        $bar = ("#" * (($script:spin % $w) + 1)).PadRight($w, ".")
        $txt = "{0,-22} [{1}]        {2,6:N0} MB" -f $label, $bar, ($done/1MB)
        $script:spin++
    }
    # Only animate on a real console; when output is piped or captured, a carriage
    # return produces a new line per tick and floods the log.
    if ($Host.UI.RawUI -and -not [Console]::IsOutputRedirected) {
        Write-Host ("`r  " + $txt) -NoNewline -ForegroundColor DarkCyan
    } elseif ($script:lastPct -ne $pct -and ($pct % 25) -eq 0) {
        Write-Host ("  " + $txt) -ForegroundColor DarkCyan
        $script:lastPct = $pct
    }
}

function Get-File ($url, $dest, $what) {
    if (Test-Path $dest) { Good "$what already here"; return }
    $tmp = "$dest.part"
    $script:spin = 0
    try {
        $req = [System.Net.HttpWebRequest]::Create($url)
        $req.UserAgent = "sarge-install"
        $res = $req.GetResponse()
        $total = $res.ContentLength
        $in  = $res.GetResponseStream()
        $outf = [System.IO.File]::Create($tmp)
        $buf = New-Object byte[] 262144
        $done = 0; $tick = 0
        while (($n = $in.Read($buf, 0, $buf.Length)) -gt 0) {
            $outf.Write($buf, 0, $n)
            $done += $n; $tick++
            if ($tick % 8 -eq 0) { Show-Bar $done $total $what }
        }
        $outf.Close(); $in.Close(); $res.Close()
        Show-Bar $done $total $what
        Write-Host ""
    } catch {
        Write-Host ""
        if (Test-Path $tmp) { Remove-Item $tmp -Force -ErrorAction SilentlyContinue }
        Stop-With "could not download $what`n  $url`n  $($_.Exception.Message)"
    }
    Move-Item $tmp $dest -Force
    Good ("{0}  ({1:N0} MB)" -f $what, ((Get-Item $dest).Length / 1MB))
}

# A spinner for the steps that have no size to measure.
function Wait-With-Spinner ($job, $label) {
    $frames = @("|", "/", "-", "\")
    $i = 0
    while ($job.State -eq "Running") {
        Write-Host ("`r  {0} {1}" -f $frames[$i % 4], $label) -NoNewline -ForegroundColor DarkCyan
        Start-Sleep -Milliseconds 120
        $i++
    }
    Write-Host ("`r" + (" " * ($label.Length + 6)) + "`r") -NoNewline
}

Write-Host ""
foreach ($l in @(
    "   ____                    ",
    "  / ___|  __ _ _ __ __ _  ___ ",
    "  \___ \ / _`` | '__/ _`` |/ _ \",
    "   ___) | (_| | | | (_| |  __/",
    "  |____/ \__,_|_|  \__, |\___|",
    "                   |___/      ")) {
    Write-Host $l -ForegroundColor DarkYellow
    Start-Sleep -Milliseconds 60
}
Write-Host ""
Write-Host "  Prompts negotiate. Sarge doesn't." -ForegroundColor White
Start-Sleep -Milliseconds 200
Write-Host "  About 2.4 GB to download, once. No GPU required." -ForegroundColor DarkGray
Write-Host ""

# ------------------------------------------------------------------ 1. the machine
Step "checking the machine"

if ($PSVersionTable.PSVersion.Major -lt 5) { Stop-With "PowerShell 5 or newer is needed." }

$py = $null
foreach ($c in @("python", "python3", "py")) {
    try {
        $v = & $c --version 2>&1
        if ($v -match "Python (\d+)\.(\d+)" -and [int]$Matches[1] -ge 3 -and [int]$Matches[2] -ge 10) { $py = $c; break }
    } catch {}
}
if (-not $py) { Stop-With "Python 3.10 or newer is needed and was not found on PATH.`n  https://www.python.org/downloads/" }
Good "Python: $(& $py --version)"

$drive = (Get-Location).Drive
if ($drive -and $drive.Free) {
    $freeGB = [math]::Round($drive.Free / 1GB, 1)
    if ($freeGB -lt $NEED_GB) { Stop-With "$freeGB GB free on $($drive.Name): - about $NEED_GB GB is needed." }
    Good "disk: $freeGB GB free"
}

# A GPU is optional. Sarge runs on CPU; the card only makes it faster.
$hasGpu = $false
try { $hasGpu = [bool](& nvidia-smi --query-gpu=name --format=csv,noheader 2>$null) } catch {}
if ($hasGpu) { Good "NVIDIA GPU found - writing will be ~6x faster" }
else         { Warn "no NVIDIA GPU - Sarge will run on CPU (judging stays fast; writing is slower)" }

# ------------------------------------------------------------------ 2. the tree
Step "the tree"

$here = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
if (-not (Test-Path (Join-Path $here "rust\replay-check.cmd"))) {
    $here = Join-Path (Get-Location).Path "Sarge"
    if (-not (Test-Path (Join-Path $here "rust\replay-check.cmd"))) {
        try { $null = Get-Command git -ErrorAction Stop } catch { Stop-With "git is needed to fetch Sarge.`n  https://git-scm.com/downloads" }
        Say "cloning $TAG into $here ..."
        # git writes progress to stderr; with ErrorActionPreference=Stop that would abort
        $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
        & git clone --depth 1 --branch $TAG $REPO $here *>&1 | Out-Null
        $ErrorActionPreference = $old
        if (-not (Test-Path (Join-Path $here "rust\replay-check.cmd"))) { Stop-With "the clone did not produce a tree at $here" }
        Good "cloned $TAG (the signed release; main is work in progress)"
    } else { Good "using the clone at $here" }
} else { Good "running inside the clone at $here" }

$bin    = Join-Path $here "organ\build\bin\Release"
$models = Join-Path $here "models"
$rustd  = Join-Path $here "rust\target\release"
New-Item -ItemType Directory -Force -Path $bin, $models, $rustd | Out-Null

# ------------------------------------------------------------------ 3. the organ
Step "the organ  (llama.cpp with Sarge compiled in)"

$server = Join-Path $bin "llama-server.exe"
if (Test-Path $server) { Good "organ already here" }
else {
    $zip = Join-Path $env:TEMP "sarge-organ.zip"
    Get-File "$REPO/releases/download/$TAG/sarge-organ-win-x64-cuda13.zip" $zip "the organ (46 MB)"
    Verify-Sha256 $zip $ORGAN_SHA256 "the organ"
    Expand-Archive -Path $zip -DestinationPath $bin -Force
    if (-not (Test-Path $server)) {
        $f = Get-ChildItem $bin -Recurse -Filter "llama-server.exe" | Select-Object -First 1
        if ($f) { Get-ChildItem $f.DirectoryName | Move-Item -Destination $bin -Force }
    }
    if (-not (Test-Path $server)) { Stop-With "llama-server.exe was not in the archive." }
    Remove-Item $zip -Force -ErrorAction SilentlyContinue
    Good "organ unpacked"
}

# the handshake ships inside the same archive; only build it if it is missing AND Rust is here
$hs = Join-Path $rustd "handshake.exe"
if (-not (Test-Path $hs)) {
    if (Test-Path (Join-Path $bin "handshake.exe")) {
        Copy-Item (Join-Path $bin "handshake.exe") $hs -Force
        Good "handshake taken from the release"
    } else {
        try { $null = Get-Command cargo -ErrorAction Stop
              Say "building the handshake with cargo (about two minutes) ..."
              Push-Location (Join-Path $here "rust"); & cargo build --release 2>&1 | Out-Null; Pop-Location
        } catch {}
        if (-not (Test-Path $hs)) { Stop-With "no handshake.exe in the release archive and no Rust toolchain to build one.`n  Install Rust (https://rustup.rs) and run this again." }
        Good "handshake built"
    }
} else { Good "handshake already here" }

# CUDA runtime: only for people who have a card and no toolkit
if ($hasGpu) {
    $dll     = Get-ChildItem $bin -Filter "cudart64*.dll" -ErrorAction SilentlyContinue | Select-Object -First 1
    $toolkit = Get-ChildItem "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA" -Directory -ErrorAction SilentlyContinue |
               Sort-Object Name -Descending | Select-Object -First 1
    if     ($dll)     { Good "CUDA runtime already beside the organ" }
    elseif ($toolkit) { Good "CUDA toolkit found: $($toolkit.Name)" }
    else {
        $zip = Join-Path $env:TEMP "cudart.zip"
        Get-File "$REPO/releases/download/$TAG/cudart-win-x64-cuda13.zip" $zip "the CUDA runtime (405 MB, one time)"
        Verify-Sha256 $zip $CUDART_SHA256 "the CUDA runtime"
        Expand-Archive -Path $zip -DestinationPath $bin -Force
        Remove-Item $zip -Force -ErrorAction SilentlyContinue
        Good "CUDA runtime unpacked"
    }
}

# ------------------------------------------------------------------ 4. the model
Step "the model  (Qwen3-4B, 2.4 GB - the long part)"

$modelPath = $null
foreach ($c in @((Join-Path $models $MODEL), "C:\llama\models\$MODEL", "C:\llama-b9213\models\$MODEL")) {
    if (Test-Path $c) { $modelPath = $c; break }
}
if ($modelPath) { Good "model already here: $modelPath" }
else {
    $modelPath = Join-Path $models $MODEL
    Say "2.4 GB, and it only happens once"
    Get-File $MODEL_URL $modelPath "the model"
    Say "checking the model (2.4 GB, this takes a few seconds)"
    Verify-Sha256 $modelPath $MODEL_SHA256 "the model"
}

# ------------------------------------------------------------------ 5. the harness
Step "the Python harness"

Push-Location $here
& $py -m pip install --disable-pip-version-check -q -e python 2>&1 |
    Where-Object { $_ -match "ERROR|error:" } | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
Pop-Location
if ((& $py -c "import sarge; print('ok')" 2>&1) -notmatch "ok") { Stop-With "the Python package did not import after install." }
Good "sarge installed"

# ------------------------------------------------------------------ 6. your paths
Step "start scripts, with your paths"

$gpuFlag = if ($hasGpu) { "-ngl 99" } else { "-ngl 0 -t 4" }
$cudaLine = ""
if ($hasGpu) {
    $tk = Get-ChildItem "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA" -Directory -ErrorAction SilentlyContinue |
          Sort-Object Name -Descending | Select-Object -First 1
    if ($tk) { $cudaLine = "set `"CUDA_PATH=$($tk.FullName)`"`r`nset `"PATH=%CUDA_PATH%\bin\x64;%CUDA_PATH%\bin;%PATH%`"" }
}

@"
@echo off
REM written by install.ps1 on $(Get-Date -Format 'yyyy-MM-dd HH:mm')
$cudaLine
"$bin\llama-server.exe" ^
  -m "$modelPath" ^
  $gpuFlag -c 32768 -np 1 -ctk q8_0 -ctv q8_0 --repeat-penalty 1.1 --repeat-last-n 256 --port 8421 %*
"@ | Set-Content (Join-Path $here "start-organ.cmd") -Encoding ASCII
Good "start-organ.cmd"

@"
@echo off
REM written by install.ps1
set "SARGE_HOME=$here"
cd /d "$here"
$py tools\console.py %*
"@ | Set-Content (Join-Path $here "start-console.cmd") -Encoding ASCII
Good "start-console.cmd"

setx SARGE_HOME $here | Out-Null
$env:SARGE_HOME = $here
Good "SARGE_HOME set"

# ------------------------------------------------------------------ 7. prove it
Step "proving it works"

# Do not start a second organ. The release review, 20 Sep: four were left running on one
# machine holding 10.8 GB between them, all fighting over :8421, and the winner changed
# three times while it was being watched. A stranger who reruns this - which is exactly
# what people do after a failure - would do the same to themselves.
$already = $false
try { $already = (Invoke-WebRequest -Uri "http://127.0.0.1:8421/health" -TimeoutSec 3 -UseBasicParsing).StatusCode -eq 200 } catch {}

if ($already) {
    Warn "an organ is already answering on :8421 - using it, not starting another"
    Say  "if it is running a different model, stop it first and run this again"
    $up = $true
} else {
$null = Start-Process -FilePath (Join-Path $here "start-organ.cmd") -WindowStyle Minimized -PassThru
$up = $false
$frames = @("|", "/", "-", "\")
for ($i = 0; $i -lt 240; $i++) {
    Write-Host ("`r  {0} loading the model into memory ..." -f $frames[$i % 4]) -NoNewline -ForegroundColor DarkCyan
    Start-Sleep -Milliseconds 900
    try { if ((Invoke-WebRequest -Uri "http://127.0.0.1:8421/health" -TimeoutSec 2 -UseBasicParsing).StatusCode -eq 200) { $up = $true; break } } catch {}
}
Write-Host ("`r" + (" " * 44) + "`r") -NoNewline
}
if (-not $up) { Stop-With "the organ did not answer on :8421 within four minutes.`n  Run start-organ.cmd yourself and read the window." }
Good "organ answering on :8421"

$fix = Join-Path $here "rust\tests\judge"
$verdict = & $hs --check $fix --file js-await-sync-bad.js --repo judge --rules (Join-Path $fix "node.sarge") 2>&1 | Out-String
if ($verdict -match "CHECKS FAILED") { Good "the check caught a known-bad file - the instrument goes red when it should" }
else {
    Write-Host "  the check did NOT catch a file it is known to catch." -ForegroundColor Red
    Write-Host "  Do not trust a pass from it until this is fixed." -ForegroundColor Red
    Stop-With "self-test failed."
}

Write-Host @"

  DONE.

    start-organ.cmd      the model, on :8421   (leave it running)
    start-console.cmd    the page, on :8420    - a folder, a task, Run

  Optional - the tutor is the one thing that leaves your machine:
    setx ANTHROPIC_API_KEY sk-ant-...

  What broke, what it got wrong, what you wish it caught:
    $REPO/discussions

"@ -ForegroundColor White
