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
set "MODE=%~2"
if "%MODE%"=="" set "MODE=embed-only"
set "FLASH_TIMEOUT_SEC=%~3"
if "%FLASH_TIMEOUT_SEC%"=="" set "FLASH_TIMEOUT_SEC=20"

if /I not "%MODE%"=="embed-only" if /I not "%MODE%"=="build" if /I not "%MODE%"=="flash" (
  echo Usage:
  echo   tools\one_click_hbc_to_f103.bat [hopping_file] [embed-only^|build^|flash] [flash_timeout_sec]
  echo Examples:
  echo   tools\one_click_hbc_to_f103.bat
  echo   tools\one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping build
  echo   tools\one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping flash 20
  popd
  pause
  exit /b 2
)

if not exist "%INPUT_HOPPING%" (
  echo Input hopping file not found: %INPUT_HOPPING%
  popd
  pause
  exit /b 1
)

set /a FLASH_TIMEOUT_SEC_NUM=%FLASH_TIMEOUT_SEC% >nul 2>nul
if errorlevel 1 (
  echo Invalid timeout seconds: %FLASH_TIMEOUT_SEC%
  popd
  pause
  exit /b 2
)
if %FLASH_TIMEOUT_SEC_NUM% LEQ 0 (
  echo Invalid timeout seconds: %FLASH_TIMEOUT_SEC%
  popd
  pause
  exit /b 2
)

if not exist "examples\asm" mkdir "examples\asm"
if not exist "examples\ir" mkdir "examples\ir"
if not exist "examples\bytecode" mkdir "examples\bytecode"

set "OUT_ASM=examples/asm/auto_f103.asm"
set "OUT_IR=examples/ir/auto_f103.ir"
set "OUT_HBC=examples/bytecode/auto_f103.hbc"
set "OUT_VM_C=firmware/stm32f103/Core/Src/vm_program_data.c"

echo [1/3] Compile hopping -^> hbc
cargo run --manifest-path "compiler/Cargo.toml" -- "%INPUT_HOPPING%" -o "%OUT_ASM%" --target stm32f403 --emit-ir "%OUT_IR%" --emit-bytecode "%OUT_HBC%"
if errorlevel 1 (
  echo Compile failed.
  popd
  pause
  exit /b 1
)

echo [2/3] Update vm_program_data.c from hbc
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\update_vm_program_data.ps1" -HbcPath "%OUT_HBC%" -OutputCPath "%OUT_VM_C%"
if errorlevel 1 (
  echo Failed to update vm_program_data.c
  popd
  pause
  exit /b 1
)

echo [3/3] Embedded bytecode refreshed.
echo Hopping : %INPUT_HOPPING%
echo HBC     : %OUT_HBC%
echo VM C    : %OUT_VM_C%

if /I "%MODE%"=="embed-only" (
  echo Mode embed-only: stop after vm_program_data.c update.
  popd
  pause
  exit /b 0
)

set "KEIL_TARGET=sram"
set "KEIL_PROJ="
for /f "delims=" %%I in ('dir /s /b "32\Fire_F103VE.uvprojx" 2^>nul') do (
  if not defined KEIL_PROJ set "KEIL_PROJ=%%~fI"
)
if not defined KEIL_PROJ (
  echo Keil project not found: 32\Fire_F103VE.uvprojx
  popd
  pause
  exit /b 3
)

set "KEIL_BUILD_LOG=%ROOT_DIR%\keil_auto_build.log"
for %%I in ("%KEIL_PROJ%") do set "KEIL_PROJ_CALL=%%~sI"
if "%KEIL_PROJ_CALL%"=="" set "KEIL_PROJ_CALL=%KEIL_PROJ%"
for %%I in ("%KEIL_BUILD_LOG%") do set "KEIL_BUILD_LOG_CALL=%%~fI"
if "%KEIL_BUILD_LOG_CALL%"=="" set "KEIL_BUILD_LOG_CALL=%KEIL_BUILD_LOG%"

set "UV4_EXE="
if exist "D:\Keil_v5\UV4\UV4.exe" set "UV4_EXE=D:\Keil_v5\UV4\UV4.exe"
if "%UV4_EXE%"=="" for %%i in (UV4.exe) do set "UV4_EXE=%%~$PATH:i"
if "%UV4_EXE%"=="" (
  echo Keil UV4 not found. Cannot build in mode: %MODE%
  popd
  pause
  exit /b 4
)

echo [Build] Keil build: %KEIL_PROJ% / target %KEIL_TARGET%
"%UV4_EXE%" -b "%KEIL_PROJ_CALL%" -t "%KEIL_TARGET%" -o "%KEIL_BUILD_LOG_CALL%"
if errorlevel 1 (
  echo Keil build failed. Log: %KEIL_BUILD_LOG%
  popd
  pause
  exit /b 5
)
echo Keil build success.

if /I "%MODE%"=="build" (
  echo Mode build: stop after Keil build.
  popd
  pause
  exit /b 0
)

set "HEX_PATH="
for /f "delims=" %%I in ('dir /s /b "32\Fire_F103VE.hex" 2^>nul') do (
  if not defined HEX_PATH set "HEX_PATH=%%~fI"
)
if not exist "%HEX_PATH%" (
  echo HEX not found after build.
  echo Build log: %KEIL_BUILD_LOG%
  popd
  pause
  exit /b 6
)
for %%I in ("%HEX_PATH%") do set "HEX_PATH_CALL=%%~sI"
if "%HEX_PATH_CALL%"=="" set "HEX_PATH_CALL=%HEX_PATH%"

set "STM32_CLI="
set "STLINK_CLI="
if exist "C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer\bin\STM32_Programmer_CLI.exe" set "STM32_CLI=C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer\bin\STM32_Programmer_CLI.exe"
if "%STM32_CLI%"=="" for %%i in (STM32_Programmer_CLI.exe) do set "STM32_CLI=%%~$PATH:i"
if exist "C:\Program Files (x86)\STMicroelectronics\STM32 ST-LINK Utility\ST-LINK_CLI.exe" set "STLINK_CLI=C:\Program Files (x86)\STMicroelectronics\STM32 ST-LINK Utility\ST-LINK_CLI.exe"
if "%STLINK_CLI%"=="" for %%i in (ST-LINK_CLI.exe) do set "STLINK_CLI=%%~$PATH:i"

if "%STM32_CLI%"=="" if "%STLINK_CLI%"=="" (
  echo No programmer CLI found.
  echo HEX: %HEX_PATH%
  popd
  pause
  exit /b 7
)

set "FLASH_TIMEOUT=0"
set "FLASH_FAILED=0"
if not "%STM32_CLI%"=="" (
  echo [Flash] Using STM32_Programmer_CLI: %STM32_CLI%
  powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$exe=$env:STM32_CLI; $hex=$env:HEX_PATH_CALL; $t=[int]$env:FLASH_TIMEOUT_SEC; " ^
    "$p=Start-Process -FilePath $exe -ArgumentList @('-c','port=SWD','-w',$hex,'-v','-rst') -PassThru -NoNewWindow; " ^
    "if($p.WaitForExit($t*1000)){ exit $p.ExitCode } else { try{$p.Kill()}catch{}; exit 124 }"
  set "STM32_RC=!errorlevel!"
  if "!STM32_RC!"=="0" (
    echo Flash success by STM32_Programmer_CLI.
    popd
    pause
    exit /b 0
  )
  if "!STM32_RC!"=="124" (
    set "FLASH_TIMEOUT=1"
    echo FLASH_TIMEOUT: STM32_Programmer_CLI timed out after %FLASH_TIMEOUT_SEC%s.
  ) else (
    set "FLASH_FAILED=1"
    echo STM32_Programmer_CLI flash failed ^(exit !STM32_RC!^).
  )
)

if not "%STLINK_CLI%"=="" (
  echo [Flash] Fallback ST-LINK_CLI: %STLINK_CLI%
  "%STLINK_CLI%" -c SWD UR -P "%HEX_PATH_CALL%" -V -Rst
  if errorlevel 1 (
    if "!FLASH_TIMEOUT!"=="1" (
      echo FLASH_TIMEOUT: primary flasher timed out and fallback failed.
      echo HEX: %HEX_PATH%
      popd
      pause
      exit /b 40
    )
    echo FLASH_FAILED: both flash methods failed.
    echo HEX: %HEX_PATH%
    popd
    pause
    exit /b 41
  )
  echo Flash success by ST-LINK_CLI.
  popd
  pause
  exit /b 0
)

if "!FLASH_TIMEOUT!"=="1" (
  echo FLASH_TIMEOUT: primary flasher timed out and no fallback is available.
  echo HEX: %HEX_PATH%
  popd
  pause
  exit /b 40
)

echo FLASH_FAILED: primary flasher failed and no fallback is available.
echo HEX: %HEX_PATH%
popd
pause
exit /b 41
