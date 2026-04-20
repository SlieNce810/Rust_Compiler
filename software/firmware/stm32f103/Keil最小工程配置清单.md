# STM32F103 最小 Keil 工程配置清单

这份清单对应当前已落地的 F103 基线工程：  
`software/32/2-液晶显示中英文（任意大小）/Project/Fire_F103VE.uvprojx`

## 0. 目标工程

- 芯片：`STM32F103VE`
- Toolchain：`ARM Compiler 5`
- 输出：`axf + hex`
- 目标：可直接 `Rebuild + Download`，并跑通 VM 点灯 + 串口/屏幕日志

## 1. 基线工程（不新建）

- 直接打开：
  - `software/32/2-液晶显示中英文（任意大小）/Project/Fire_F103VE.uvprojx`
- 保留该工程自带：
  - CMSIS/HAL 库
  - `User` 目录 BSP（`led/usart/lcd/font`）
  - 启动文件 `startup_stm32f103xe.s`

## 2. Include Path（当前应包含）

`Options for Target -> C/C++ -> Include Paths` 至少有：

- `..\Libraries\CMSIS\Include`
- `..\Libraries\CMSIS\Device\ST\STM32F1xx\Include`
- `..\Libraries\STM32F1xx_HAL_Driver\Inc`
- `..\User`
- `..\User\led`
- `..\..\..\firmware\common`  （新增，供 `vm_core.h/vm_hal.h/vm_program.h`）
- `..\..\..\firmware\stm32f103\Core\Inc`（新增，供 `bootloader_hbc.h`）

## 3. 宏定义（当前）

`Options for Target -> C/C++ -> Define`：

- `USE_HAL_DRIVER`
- `STM32F103xE`

## 4. 本项目新增源码（必须）

在工程中新增一组 `VM` 文件：

- `..\..\..\firmware\common\vm_core.c`
- `..\..\..\firmware\stm32f103\Core\Src\vm_program_data.c`
- `..\..\..\firmware\stm32f103\Core\Src\bootloader_hbc.c`

并将示例主入口改为 VM 运行入口：

- `..\User\main.c`

## 5. 关键行为说明

- `main.c` 中实现 `hal_*` 接口（`hal_init/hal_log_int/hal_led_write/hal_millis`）
- 日志输出路径：
  - 串口：`DEBUG_USART_Config` + `printf`
  - 屏幕：`ILI9341_DispString_EN`
- 上电升级窗口：
  - 串口监听升级包（`magic/version/length/crc32/payload`）
  - 成功写入 Flash 后优先加载动态字节码
  - 日志键：`bl_update`（升级结果）、`bl_slot`（0=内置,1=Flash）
- 心跳与延时依赖 HAL Tick：
  - `hal_millis -> HAL_GetTick()`
  - `hal_delay_ms -> HAL_Delay()`

## 6. 下载前快速自检

- 编译无 `cannot open source input file`（尤其是中文路径失效）
- 无 `undefined` 到 `vm_*` / `g_vm_program*`
- 无重复中断符号冲突（如 `SysTick_Handler`）

## 7. 上电验收口径

下载成功后应看到：

- 串口输出（USART1，115200）：
  - `[VM] vm_boot = 1`
  - `[VM] vm_loaded = 1`
  - `[VM] vm_retval = ...`
  - `[VM] led_on = ...`
  - `[VM] tick_ms = ...`
  - 异常时：`[VM] vm_load_err = ...` 或 `[VM] vm_run_err = ...`
- LCD 同步滚动输出上述日志
- 绿色 LED（PB0）随 `retval != 0` 周期翻转

## 8. 常见问题

- 屏幕黑屏但串口有日志：优先查 FSMC 硬件连线/背光控制。
- LED 不亮：检查 `User/led/bsp_led.h` 的 `LED2` 定义是否与板卡一致。
- 串口乱码：串口工具设为 `115200 8N1`，确认接收 `PA9(TX)`。

## 9. RAM 紧张裁剪开关（本轮新增）

为了解决 `L6406E/L6407E`，当前工程默认启用“VM 最小化 LCD 模式”：

- 文件：`software/32/2-液晶显示中英文（任意大小）/User/lcd/bsp_ili9341_lcd.c`
- 开关：`VM_LCD_MINIMAL_PROFILE`（默认 `1`）
- 效果：
  - `zoomBuff` 从 `16384` 降到 `512`
  - `zoomTempBuff` 从 `1024` 降到 `256`
  - 保留 `ILI9341_DispString_EN` 日志路径，不影响 VM 日志显示

如确需恢复“任意大小字体演示”，可将该开关改为 `0`，但需自行评估 RAM 占用并重新通过链接验收。
