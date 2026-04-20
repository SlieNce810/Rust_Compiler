param(
    [Parameter(Mandatory = $true)]
    [string]$HbcPath,
    [Parameter(Mandatory = $true)]
    [string]$Port,
    [int]$BaudRate = 115200
)

$ErrorActionPreference = "Stop"

function Get-Crc32([byte[]]$data) {
    $crc = 0xFFFFFFFFL
    foreach ($b in $data) {
        $crc = ($crc -bxor [int64]$b) -band 0xFFFFFFFFL
        for ($i = 0; $i -lt 8; $i++) {
            if (($crc -band 1L) -ne 0) {
                $crc = (($crc -shr 1) -bxor 0xEDB88320L) -band 0xFFFFFFFFL
            } else {
                $crc = ($crc -shr 1) -band 0xFFFFFFFFL
            }
        }
    }
    return [uint32](($crc -bxor 0xFFFFFFFFL) -band 0xFFFFFFFFL)
}

if (!(Test-Path -LiteralPath $HbcPath)) {
    throw "HBC file not found: $HbcPath"
}

if ([string]::IsNullOrWhiteSpace($Port)) {
    throw "Serial port is required, e.g. COM3."
}

$payload = [System.IO.File]::ReadAllBytes($HbcPath)
if ($payload.Length -eq 0) {
    throw "HBC payload is empty."
}

$maxBytes = 61440
if ($payload.Length -gt $maxBytes) {
    throw "HBC too large ($($payload.Length)), max is $maxBytes bytes."
}

$crc = Get-Crc32 $payload
$packet = New-Object System.Collections.Generic.List[byte]

# magic "HUP1"
$packet.AddRange([byte[]](0x48, 0x55, 0x50, 0x31))
# version
$packet.Add(0x01)
# flags (reserved)
$packet.Add(0x00)
# payload len (u32 LE)
$packet.AddRange([BitConverter]::GetBytes([uint32]$payload.Length))
# crc32 (u32 LE)
$packet.AddRange([BitConverter]::GetBytes([uint32]$crc))
# payload
$packet.AddRange($payload)

$serial = New-Object System.IO.Ports.SerialPort $Port, $BaudRate, "None", 8, "One"
$serial.WriteTimeout = 10000
$serial.ReadTimeout = 1000

Write-Host "Open serial:" $Port "@" $BaudRate
$availablePorts = [System.IO.Ports.SerialPort]::GetPortNames()
if ($availablePorts.Count -eq 0) {
    throw "No serial ports detected."
}
if (!($availablePorts -contains $Port)) {
    throw "Serial port '$Port' not found. Available: $($availablePorts -join ', ')"
}

$serial.Open()
try {
    Start-Sleep -Milliseconds 200
    $bytes = $packet.ToArray()
    $serial.Write($bytes, 0, $bytes.Length)
    $serial.BaseStream.Flush()
    Write-Host "Sent packet bytes:" $bytes.Length
    Write-Host "Payload bytes:" $payload.Length
    Write-Host "CRC32:" ("0x{0:X8}" -f $crc)
    Write-Host "Tip: reset device then check 'bl_update=6' log."
}
finally {
    if ($serial.IsOpen) {
        $serial.Close()
    }
}
