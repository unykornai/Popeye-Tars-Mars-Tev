# health-check.ps1 - Check health of devnet nodes
# Usage: .\scripts\health-check.ps1

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$LogDir = Join-Path $ProjectRoot "logs"
$DataDir = Join-Path $ProjectRoot "dev_data"

Write-Host "=== Unykorn Devnet Health Check ===" -ForegroundColor Cyan
Write-Host ""

# Check processes
Write-Host "PROCESSES:" -ForegroundColor Yellow
$Processes = Get-Process -Name "unykorn" -ErrorAction SilentlyContinue
if ($Processes) {
    foreach ($P in $Processes) {
        $Mem = [math]::Round($P.WorkingSet64 / 1MB, 2)
        $Cpu = $P.CPU
        Write-Host "  PID $($P.Id): Memory=${Mem}MB CPU=${Cpu}s" -ForegroundColor Green
    }
} else {
    Write-Host "  No unykorn processes running" -ForegroundColor Red
}

# Check log files
Write-Host ""
Write-Host "LOG FILES:" -ForegroundColor Yellow
$Nodes = @("node-a", "node-b", "node-c")
foreach ($Node in $Nodes) {
    $LogFile = Join-Path $LogDir "$Node.log"
    if (Test-Path $LogFile) {
        $Size = (Get-Item $LogFile).Length
        $LastLine = Get-Content $LogFile -Tail 1 -ErrorAction SilentlyContinue
        Write-Host "  $Node.log: ${Size} bytes" -ForegroundColor Green
        if ($LastLine) {
            Write-Host "    Last: $LastLine" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "  $Node.log: not found" -ForegroundColor DarkYellow
    }
}

# Check data directories
Write-Host ""
Write-Host "DATA DIRECTORIES:" -ForegroundColor Yellow
$SubDirs = @("blocks", "state")
foreach ($SubDir in $SubDirs) {
    $Path = Join-Path $DataDir $SubDir
    if (Test-Path $Path) {
        $Files = Get-ChildItem $Path -File -ErrorAction SilentlyContinue
        Write-Host "  $SubDir/: $($Files.Count) files" -ForegroundColor Green
    } else {
        Write-Host "  $SubDir/: not found" -ForegroundColor DarkYellow
    }
}

# Check ports
Write-Host ""
Write-Host "PORTS:" -ForegroundColor Yellow
$Ports = @(9001, 9002, 9003)
foreach ($Port in $Ports) {
    $Conn = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
    if ($Conn) {
        Write-Host "  :$Port - LISTENING (PID $($Conn.OwningProcess))" -ForegroundColor Green
    } else {
        Write-Host "  :$Port - not listening" -ForegroundColor DarkYellow
    }
}

Write-Host ""
Write-Host "=== End Health Check ===" -ForegroundColor Cyan
