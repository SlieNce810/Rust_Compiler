# Bootloader 输入清单（已落地版）

## 1. 芯片与开发板

### STM32F103
- 芯片型号：`STM32F103VE`
- 开发板型号：野火 F103VE（液晶示例工程基线）
- 核心架构：`Cortex-M3`
- Flash 容量：`512KB`
- RAM 容量：`64KB`

### ESP32S3
- 芯片型号：`ESP32-S3`
- 开发板型号：ESP32-S3-DevKitC（默认）
- 核心架构：`Xtensa`
- Flash 容量：按模块配置（常见 4MB/8MB）
- RAM：按芯片与配置

## 2. Bootloader 方案

- 方案类型：**应用内轻量 bootloader（A 方案） + 固件自动烧录（B 方案）双轨**
- A 方案：
  - 上电后短窗口监听 UART 升级包
  - 校验成功后把 `.hbc` 写入 Flash 数据区
  - 运行时优先加载 Flash 中的动态字节码，否则回退内置 `vm_program_data.c`
- B 方案：
  - 脚本执行 `hopping -> hbc -> vm_program_data.c -> Build -> Flash`

## 3. 下载链路

- 第一阶段接口：`UART`
- 包协议（A 方案）：
  - `magic(4) + version(1) + flags(1) + length(4LE) + crc32(4LE) + payload`
  - `magic = "HUP1"`，`version = 1`
- 默认串口参数：`115200 8N1`

## 4. 启动与控制引脚

### STM32F103
- 串口：`USART1 (PA9/PA10)`
- LED：`PB0(LED2)`
- 屏幕：ILI9341 FSMC

### ESP32S3
- 串口：`UART0`（USB CDC / UART）
- LED：`GPIO2`（默认）

## 5. Flash 布局

### STM32F103（当前实现）
- 动态字节码数据区：`0x08070000`
- 元数据区：`0x0807F000`
- page size：`2KB`
- payload 最大：`61440 bytes`

### ESP32S3（当前实现）
- 使用名为 `hbc_slot` 的 DATA 分区（需在分区表提供）
- 分区内：
  - `offset 0`：元数据
  - `offset 4096`：payload
- payload 最大：`16384 bytes`（当前实现上限）

## 6. 更新策略

- 当前：单槽写入 + 完整性校验 + 内置程序回退
- 校验失败：保持运行内置字节码（不会加载损坏动态包）
- 后续可扩展：双槽 A/B 与真正版本回滚

## 7. 运行模型

- 下载对象：`.hopping` 编译后的 `.hbc`
- VM 执行优先级：
  1. 动态 Flash 字节码（有效）
  2. 内置 `vm_program_data.c`

## 8. 安全与校验

- 已实现：
  - `magic`
  - `version`
  - `length` 上限检查
  - `crc32`
- 暂未实现：
  - 签名校验
  - 设备绑定

## 9. 对应脚本

### F103
- B 自动烧录：`tools/one_click_hbc_to_f103.bat`
- A UART 下载：`tools/uart_hbc_update_f103.bat`
- UART 发送工具：`tools/send_hbc_to_f103.ps1`

### ESP32S3
- B 自动烧录：`tools/one_click_hbc_to_esp32.bat`
- A UART 下载：`tools/uart_hbc_update_esp32.bat`
