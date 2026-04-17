# 端侧Demo运行手册（STM32F407 + ESP32S3）

> 推荐先阅读：
> 1) `Docs/03_实现设计/端侧解释器自顶向下总学习笔记.md`  
> 2) `Docs/03_实现设计/MCU解释器V1设计.md`

## 1. 目标

本手册用于复现本轮端侧解释器演示闭环：

`Hopping源码 -> 主机编译产出 main.hbc -> 固件内置字节码 -> 真机解释执行 -> 点灯 + 串口日志`

## 2. 目录与关键文件

- 主机编译器：`software/compiler`
- 示例源码：`software/examples/source/main.hopping`
- 产物目录：`software/examples/ir`、`software/examples/bytecode`
- 共享 VM：`software/firmware/common`
- STM32 工程：`software/firmware/stm32f407`
- ESP32 工程：`software/firmware/esp32s3`

## 3. 主机侧生成产物

在 `software/compiler` 执行：

```bat
cargo check
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403 --emit-demo-artifacts
```

预期产物：

- `software/examples/ir/main.ir`
- `software/examples/bytecode/main.hbc`

## 4. 将 main.hbc 内置到固件

当前 V1 使用“固件内置字节码”。做法：

1. 读取 `software/examples/bytecode/main.hbc`
2. 转成 `uint8_t` 数组
3. 覆盖以下两个文件中的 `g_vm_program`：
   - `software/firmware/stm32f407/Core/Src/vm_program_data.c`
   - `software/firmware/esp32s3/main/vm_program_data.c`

说明：下一轮接入动态下载后将通过 `vm_load_program()` 直接装载，不再依赖编译期内置。

## 5. STM32F407 构建与烧录

### 5.1 最小工程前置

当前仓库保留的是最小可读工程骨架。若本机缺以下文件，需要先由 CubeMX/CubeIDE 工程补齐：

- `STM32F407VGTx_FLASH.ld`
- `startup_stm32f407xx.s`
- `system_stm32f4xx.c`
- `STM32F4xx_HAL_Driver` 与 `CMSIS` 依赖

### 5.2 构建与下载

- 构建方式：`make` 或 `STM32CubeIDE`
- 下载工具：`STM32CubeProgrammer`
- 串口：`USART2`（115200）

### 5.3 验收项

- 上电日志包含：`vm_boot=1`、`vm_loaded=1`
- 运行中可见 `led_on` 变化与 `tick_ms`
- 异常时输出 `vm_load_err` 或 `vm_run_err`

## 6. ESP32S3 构建与烧录

在 `software/firmware/esp32s3` 执行：

```bash
idf.py set-target esp32s3
idf.py build
idf.py -p <PORT> flash monitor
```

### 6.1 验收项

- 启动日志包含：`vm_boot=1`、`vm_loaded=1`
- 运行中打印：`vm_retval`、`led_on`、`tick_ms`
- LED 行为与 STM32 一致（忽略时间戳）

## 7. 一致性检查

同一份 `main.hbc` 在两平台执行时，比较日志键序列：

- `vm_boot`
- `vm_loaded`
- `vm_retval`
- `led_on`
- `tick_ms`

允许差异：时间戳值、硬件时钟精度。

## 8. 常见问题

- `vm_load_err=2/3`：字节码头 `magic/version` 不匹配
- `vm_run_err=12`：跳转目标越界（字节码损坏或导出错误）
- LED 不变化但无错误：`vm_retval` 始终为 `0`，需检查示例逻辑
