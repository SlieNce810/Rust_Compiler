# STM32F103 点灯 + 屏幕日志接入说明

本轮 F103 以示例工程为主基线：

- `software/32/2-液晶显示中英文（任意大小）/Project/Fire_F103VE.uvprojx`

配套配置清单见：

- `Keil最小工程配置清单.md`

## 1. 已接入能力

- VM 执行结果驱动 LED 翻转（默认 `LED2/PB0`）
- K1/K2 按键参与频率控制（`K1=PA0`、`K2=PC13`）
- 日志双通道输出：
  - 串口：USART1 (`PA9/PA10`)
  - 屏幕：ILI9341（FSMC 并口）
- 统一日志键名：`vm_boot`、`vm_loaded`、`vm_retval`、`led_on`、`tick_ms`（异常 `vm_load_err` / `vm_run_err`）

## 2. 关键源码位置

- VM 公共层：
  - `software/32/firmware/common/vm_core.c`
  - `software/32/firmware/common/vm_core.h`
  - `software/32/firmware/common/vm_hal.h`
  - `software/32/firmware/common/vm_program.h`
- F103 字节码数据：
  - `software/32/firmware/stm32f103/Core/Src/vm_program_data.c`
- F103 动态升级（UART + Flash）：
  - `software/firmware/stm32f103/Core/Inc/bootloader_hbc.h`
  - `software/firmware/stm32f103/Core/Src/bootloader_hbc.c`
- 示例工程主入口（已改为 VM 运行循环）：
  - `software/32/2-液晶显示中英文（任意大小）/User/main.c`

## 3. 工程集成要点（Fire_F103VE）

- 在 `Fire_F103VE.uvprojx` 中加入：
  - `..\..\firmware\common\vm_core.c`
  - `..\..\firmware\stm32f103\Core\Src\vm_program_data.c`
- `IncludePath` 增加：
  - `..\..\firmware\common`
- 继续复用示例工程自带 HAL/CMSIS 与 BSP（`User/led`、`User/usart`、`User/lcd`、`User/font`）

## 4. 烧录后验收口径

- 串口持续输出 VM 关键日志（含异常分支日志）
- LCD 按行滚动显示同样日志，不黑屏
- 绿灯按 `vm_retval != 0` 周期翻转

## 5. 双轨升级命令

- B 自动构建+下载（内置字节码）：
  - `tools/one_click_hbc_to_f103.bat [examples/source/xxx.hopping] [embed-only|build|flash] [flash_timeout_sec]`
- A 真动态下载（UART 包）：
  - `tools/uart_hbc_update_f103.bat [examples/source/xxx.hopping] <COMx> [115200]`

> A 方案上电后会有短暂接收窗口，日志字段 `bl_update`、`bl_slot` 可用于判断是否加载到动态字节码。

### 5.1 脚本行为说明（稳定性增强）

- `one_click_hbc_to_f103.bat`
  - 第二参数 `mode`：`embed-only|build|flash`（默认 `embed-only`）。
  - 第三参数 `flash_timeout_sec`（默认 20）仅在 `flash` 模式生效。
  - `embed-only`：生成 `.hbc` 并写入 `vm_program_data.c`。
  - `build`：在 `embed-only` 基础上执行 Keil 构建。
  - `flash`：在 `build` 基础上执行自动烧录。
  - 优先使用 `STM32_Programmer_CLI` 烧录。
  - 默认 20 秒超时（可用第三参数覆盖），超时会输出 `FLASH_TIMEOUT` 并自动回退到 `ST-LINK_CLI`。
  - 两种 CLI 都失败时，会输出 `HEX` 路径用于手工烧录。
- `uart_hbc_update_f103.bat`
  - 串口参数改为必填：不再默认 `COM3`。
  - 若未传串口，会报 `COM_PORT_REQUIRED` 并列出当前可用串口。

### 5.2 K1/K2 调频点灯命令

- 编译并更新内置字节码（不构建）：
  - `tools/one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping embed-only`
- 编译并构建：
  - `tools/one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping build`
- 编译、构建并烧录：
  - `tools/one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping flash 20`
- 使用 bootloader 热更新（UART）：
  - `tools/uart_hbc_update_f103.bat examples/source/from0_f103_led.hopping COM5 115200`

验收日志：
- `vm_loaded = 1`
- `bl_update = 6`（热更新成功）
- `bl_slot = 1`（从 Flash 运行动态字节码）

## 6. RAM 裁剪边界（本轮新增）

当前 F103 基线按“最小化运行集”配置，目标是稳定保留以下能力：

- VM 执行
- LED 翻转
- UART 日志
- LCD 文本日志（`ILI9341_DispString_EN`）

默认关闭/弱化的是液晶演示中的大内存缩放路径（任意大小字体缓存）。若要恢复该能力，请调整
`software/32/2-液晶显示中英文（任意大小）/User/lcd/bsp_ili9341_lcd.c` 的
`VM_LCD_MINIMAL_PROFILE` 开关，并重新验证链接内存。
