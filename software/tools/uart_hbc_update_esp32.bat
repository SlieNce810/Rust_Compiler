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
if "%INPUT_HOPPING%"=="" set "INPUT_HOPPING=examples/source/esp32_blink.hopping"
set "COM_PORT=%~2"
if "%COM_PORT%"=="" set "COM_PORT=COM4"
set "BAUD=%~3"
if "%BAUD%"=="" set "BAUD=115200"
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

set "OUT_ASM=examples/asm/uart_esp32.asm"
set "OUT_IR=examples/ir/uart_esp32.ir"
set "OUT_HBC=examples/bytecode/uart_esp32.hbc"

echo [1/2] Compile hopping -> hbc
cargo run --manifest-path "compiler/Cargo.toml" -- "%INPUT_HOPPING%" -o "%OUT_ASM%" --target esp32 --emit-ir "%OUT_IR%" --emit-bytecode "%OUT_HBC%"
if errorlevel 1 (
  echo Compile failed.
  popd
  pause
  exit /b 1
)

echo [2/2] Send hbc packet via UART %COM_PORT% @ %BAUD%
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\send_hbc_to_f103.ps1" -HbcPath "%OUT_HBC%" -Port "%COM_PORT%" -BaudRate %BAUD%
if errorlevel 1 (
  echo UART send failed.
  echo Tip: check COM port and bootloader receive window.
  popd
  pause
  exit /b 1
)

echo Done. Reset board and observe bl_update/bl_slot logs.
popd
pause
exit /b 0
