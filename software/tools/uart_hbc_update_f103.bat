@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"
set "ROOT_DIR=%SCRIPT_DIR%.."

pushd "%ROOT_DIR%"
if errorlevel 1 (
  echo Failed to enter project root: %ROOT_DIR%
  pause
  exit /b 1
)

set "INPUT_HOPPING=%~1"
if "%INPUT_HOPPING%"=="" set "INPUT_HOPPING=examples/source/from0_f103_led.hopping"
set "COM_PORT=%~2"
set "BAUD=%~3"
if "%BAUD%"=="" set "BAUD=115200"
if "%COM_PORT%"=="" (
  echo COM_PORT_REQUIRED: please pass COM port, e.g. COM4
  powershell -NoProfile -ExecutionPolicy Bypass -Command "$ports=[System.IO.Ports.SerialPort]::GetPortNames(); if($ports.Count -eq 0){Write-Host 'Available ports: (none)'} else {Write-Host ('Available ports: ' + ($ports -join ', '))}"
  popd
  pause
  exit /b 11
)
set "COM_PORT=%COM_PORT:"=%"
set "COM_PORT=%COM_PORT: =%"
set "BAUD=%BAUD:"=%"
set "BAUD=%BAUD: =%"
if not exist "%INPUT_HOPPING%" (
  echo Input hopping file not found: %INPUT_HOPPING%
  popd
  pause
  exit /b 1
)
if "%BAUD%"=="" (
  echo Invalid baud rate: %BAUD%
  popd
  pause
  exit /b 1
)
set /a BAUD_NUM=%BAUD% >nul 2>nul
if errorlevel 1 (
  echo Invalid baud rate: %BAUD%
  popd
  pause
  exit /b 1
)

if not exist "examples\asm" mkdir "examples\asm"
if not exist "examples\ir" mkdir "examples\ir"
if not exist "examples\bytecode" mkdir "examples\bytecode"

set "OUT_ASM=examples/asm/uart_f103.asm"
set "OUT_IR=examples/ir/uart_f103.ir"
set "OUT_HBC=examples/bytecode/uart_f103.hbc"

echo [1/3] Compile hopping -^> hbc
cargo run --manifest-path "compiler/Cargo.toml" -- "%INPUT_HOPPING%" -o "%OUT_ASM%" --target stm32f403 --emit-ir "%OUT_IR%" --emit-bytecode "%OUT_HBC%"
if errorlevel 1 (
  echo Compile failed.
  popd
  pause
  exit /b 1
)

echo [2/3] Build hot-update packet and send by UART
echo Send hbc via UART %COM_PORT% @ %BAUD%
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\send_hbc_to_f103.ps1" -HbcPath "%OUT_HBC%" -Port "%COM_PORT%" -BaudRate %BAUD%
if errorlevel 1 (
  echo UART send failed.
  echo Tip: check COM port and that board is in boot update window.
  popd
  pause
  exit /b 12
)

echo [3/3] Bootloader hot-update send complete.
echo Expect log:
echo   bl_update = 6
echo   bl_slot   = 1
echo If not received, reset board and re-run within boot window.
popd
pause
exit /b 0
