# 嵌入式简化编译器（Rust）

本项目实现了一个最小可运行编译器流程：

- 词法分析（Lexer）
- 语法分析（递归下降 Parser）
- 语义检查（声明与类型检查）
- IR 生成（TAC 三地址码）
- 后端汇编模板生成（`stm32f403` / `esp32`）

项目根目录：`E:\02_competition\Rust_Compiler\software`

## 1. 目录说明

- `compiler/`：编译器源码（Rust）
- `examples/source/main.hopping`：示例输入源码
- `examples/asm/main_stm32.asm`：示例输出（STM32 风格）
- `examples/asm/main_esp32.asm`：示例输出（ESP32 风格）
- `examples/ir/main.ir`：可读 IR 产物
- `examples/bytecode/main.hbc`：端侧执行字节码
- `firmware/`：STM32F407 与 ESP32S3 端侧解释器工程
- `Docs/03_实现设计/MCU解释器V1设计.md`：字节码与 VM 规范

## 2. 环境要求

你需要：

1. Rust（stable）
2. MSVC C++ 工具链（提供 `link.exe`）
3. Windows SDK（提供 `kernel32.lib`）

本机当前安装路径：

- `C:\BuildTools\...`（Build Tools）

## 3. 每次打开新终端先做这一步

先注入 MSVC 编译环境变量（非常关键）：

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
```

如果不执行这步，`cargo run` 可能报 `link.exe` 或 `kernel32.lib` 相关错误。

## 4. 如何编译并跑一个 Demo

### 4.1 进入编译器目录

```bat
cd /d E:\02_competition\Rust_Compiler\software\compiler
```

### 4.2 生成 STM32 风格汇编

```bat
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403
```

### 4.3 生成 ESP32 风格汇编

```bat
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_esp32.asm --target esp32
```

### 4.4 查看输出结果

```bat
type E:\02_competition\Rust_Compiler\software\examples\asm\main_stm32.asm
type E:\02_competition\Rust_Compiler\software\examples\asm\main_esp32.asm
```

### 4.5 同时落盘 IR 与字节码

```bat
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403 --emit-ir ..\examples\ir\main.ir --emit-bytecode ..\examples\bytecode\main.hbc
```

或直接使用固定演示产物路径：

```bat
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403 --emit-demo-artifacts
```

说明：

- `--emit-ir` 输出可读 IR 文本
- `--emit-bytecode` 输出二进制字节码文件（`.hbc`）
- `--emit-demo-artifacts` 固定输出到 `examples/ir/main.ir` 与 `examples/bytecode/main.hbc`
- `--ai-explain-error` 编译失败时在终端输出 AI 修复建议（不再默认写失败 md）
- `--ai-only-on-error` 仅在失败时触发 AI；成功编译时跳过 AI 报告生成
- `--ai-report` 指定成功编译时 AI 报告输出路径（IR 讲解 + 测试建议）
- `--ai-provider` 指定 AI Provider：`mock|local|deepseek`（`cloud` 作为 `deepseek` 兼容别名）
- `--ai-api-key` 通过 CLI 传入 DeepSeek key（优先级最高）
- DeepSeek 鉴权环境变量：`HOPPING_AI_API_KEY`（或兼容 `DEEPSEEK_API_KEY`）
- 可选环境变量：
  - `HOPPING_AI_BASE_URL`（默认 `https://api.deepseek.com`）
  - `HOPPING_AI_MODEL`（默认 `deepseek-chat`）

## 5. 最快复现命令（复制即用）

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d E:\02_competition\Rust_Compiler\software\compiler
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_esp32.asm --target esp32
```

## 5.1 AI 辅助示例（终端建议 + 成功报告）

### 成功编译 + IR 讲解

```bat
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403 --ai-report ..\examples\notes\ai_report_success.md --ai-provider mock
```

### 失败编译 + 错误解释

```bat
cargo run -- ..\examples\source\invalid_v1_float.hopping -o ..\examples\asm\invalid.asm --target stm32f403 --ai-explain-error --ai-provider mock
```

### DeepSeek 真模型错误修复建议（失败时，终端输出）

```bat
cargo run -- ..\examples\source\invalid_v1_float.hopping -o ..\examples\asm\invalid.asm --target stm32f403 --ai-explain-error --ai-provider deepseek --ai-api-key sk-xxxx
```

### 只在失败时启用 AI（成功不生成报告）

```bat
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403 --ai-only-on-error --ai-explain-error --ai-provider deepseek --ai-api-key sk-xxxx
```

### 成功编译时生成 DeepSeek IR 报告

```bat
cargo run -- ..\examples\source\main.hopping -o ..\examples\asm\main_stm32.asm --target stm32f403 --ai-report ..\examples\notes\ai_report_success_deepseek.md --ai-provider deepseek --ai-api-key sk-xxxx
```

## 6. Demo 源码（当前）

`examples/source/main.hopping` 内容是一个简单函数，包含：

- 变量声明与赋值
- `if/else`
- `while`
- `return`

它用于验证完整编译流程是否跑通。

## 7. 当前后端状态（重要）

当前后端已经具备：

- 基本函数栈帧生成
- 局部变量落栈读写
- 条件/跳转/返回路径生成
- 目标风格区分（STM32 vs ESP32）

但仍是“教学/模板级后端”，尚未完成：

- 严格 ABI（函数参数传递、调用约定）
- 完整寄存器分配
- 可直接汇编链接到真实芯片固件的全链路

## 8. 常见问题

### Q1: 报错 `link.exe not found`
先运行：

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
```

### Q2: 报错 `kernel32.lib` 找不到
说明 Windows SDK 环境未正确加载，仍然先运行 `vcvars64.bat`，再执行 `cargo run`。

### Q3: 报错 `unexpected char: \u{feff}`
通常是源码文件 BOM 问题。当前 lexer 已兼容 BOM；若仍出现，确保源码使用 UTF-8 或 ASCII。

## 9. 点灯 Demo（Rust 编译器链路）

下面两个 demo 都是通过当前 Rust 编译器生成，不依赖外部 C 工程。

### 9.1 生成 STM32 点灯汇编

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d E:\02_competition\Rust_Compiler\software\compiler
cargo run -- ..\examples\source\stm32_blink.hopping -o ..\examples\asm\stm32_blink.asm --target stm32f403
```

### 9.2 生成 ESP32 点灯汇编

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d E:\02_competition\Rust_Compiler\software\compiler
cargo run -- ..\examples\source\esp32_blink.hopping -o ..\examples\asm\esp32_blink.asm --target esp32
```

### 9.3 Demo 文件说明

- 输入源码：
  - `examples/source/stm32_blink.hopping`
  - `examples/source/esp32_blink.hopping`
- 输出汇编：
  - `examples/asm/stm32_blink.asm`
  - `examples/asm/esp32_blink.asm`
- GPIO 替换指引：
  - `examples/notes/gpio_interface_todo.md`

> 当前 GPIO 是占位接口变量，目的是先跑通编译链路。你可以按开发板手册把占位变量替换成真实寄存器/HAL 调用。
> - 占位变量：`stm32_gpio_mode_reg` / `esp32_gpio_enable_reg`
> - 真实寄存器：`GPIOA_MODER` / `GPIO_ENABLE_REG`
> - 真实 HAL 调用：`HAL_GPIO_Init` / `HAL_GPIO_Init`

## 10. 端侧解释器 Demo（STM32F407 + ESP32S3）

### 10.1 共享核心与平台目录

- 共享 VM：`firmware/common/vm_core.c`
- STM32F407：`firmware/stm32f407`
- STM32F103（指南者，点灯+LCD日志）：`firmware/stm32f103`
- ESP32S3：`firmware/esp32s3`

### 10.2 端侧日志口径

两平台统一输出以下键值：

- `vm_boot`
- `vm_loaded`
- `vm_retval`
- `led_on`
- `tick_ms`
- `vm_load_err` / `vm_run_err`

### 10.3 烧录方式（本轮）

- STM32F407：`STM32CubeProgrammer`
- ESP32S3：`idf.py flash monitor`（底层 `esptool`）

详细步骤见：`Docs/03_实现设计/端侧Demo运行手册.md`

## 10.4 双轨升级脚本（新增）

### STM32F103

- B 自动构建+下载（内置字节码）：
  - `tools\one_click_hbc_to_f103.bat [examples/source/xxx.hopping] [embed-only|build|flash] [flash_timeout_sec]`
- A 真动态下载（UART 包下发）：
  - `tools\uart_hbc_update_f103.bat [examples/source/xxx.hopping] <COMx> [115200]`
- UART 发包脚本：
  - `tools\send_hbc_to_f103.ps1 -HbcPath examples/bytecode/xxx.hbc -Port COM3 -BaudRate 115200`

### ESP32S3

- A 真动态下载（UART 包下发）：
  - `tools\uart_hbc_update_esp32.bat [examples/source/xxx.hopping] [COMx] [115200]`

> A 方案统一使用升级包头：`magic(HUP1) + version + length + crc32 + payload`。

> 当前 `tools` 目录可用批处理脚本为：`one_click_hbc_to_f103.bat`、`uart_hbc_update_f103.bat`、`uart_hbc_update_esp32.bat`。

### 10.5 脚本行为（当前版本）

- `tools\one_click_hbc_to_f103.bat`
  - 第二参数可选：`mode`，支持 `embed-only|build|flash`，默认 `embed-only`。
  - 第三参数可选：`flash_timeout_sec`，默认 `20` 秒（仅 `flash` 模式生效）。
  - `embed-only`：只做 `.hopping -> .hbc -> vm_program_data.c`。
  - `build`：在 `embed-only` 基础上执行 Keil build。
  - `flash`：在 `build` 基础上自动烧录。
  - 烧录优先使用 `STM32_Programmer_CLI`。
  - 超时会输出 `FLASH_TIMEOUT`，并自动回退到 `ST-LINK_CLI`（若可用）。
  - 若最终仍失败，会打印 `HEX` 路径用于手工烧录。

- `tools\uart_hbc_update_f103.bat`
  - `COMx` 是必填参数（不再默认 `COM3`）。
  - 未传串口时会输出 `COM_PORT_REQUIRED` 并列出当前可用串口。

- `warning: constant OP_NOP is never used`
  - 这是 Rust 编译器告警，不影响 `.hbc` 生成与 UART 发包流程。

## 10.6 F103 K1/K2 调频点灯 Demo（当前可用）

- Hopping 内建函数（新增）：
  - `key1_read()`：读 K1，按下返回 `1`，否则 `0`
  - `key2_read()`：读 K2，按下返回 `1`，否则 `0`
  - `sleep_ms(int)`：毫秒延时
- 按键引脚（指南者）：
  - `K1 = PA0`
  - `K2 = PC13`

### 编译并写入内置字节码

```bat
cd /d E:\02_competition\Rust_Compiler\software
tools\one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping embed-only
```

### 编译 + Keil 构建（不烧录）

```bat
tools\one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping build
```

### 编译 + 构建 + 烧录

```bat
tools\one_click_hbc_to_f103.bat examples/source/from0_f103_led.hopping flash 20
```

### 真 bootloader 热更新（UART）

```bat
tools\uart_hbc_update_f103.bat examples/source/from0_f103_led.hopping COM5 115200
```

成功日志口径（端侧串口/LCD）：
- `bl_update = 6`：升级包校验并写入成功
- `bl_slot = 1`：本次从 Flash 动态字节码运行

## 11. 文件结构（当前）
``` 
E:.
├─compiler
│  ├─src
│  │  ├─ast.rs
│  │  ├─backend.rs
│  │  ├─bytecode.rs
│  │  ├─ir.rs
│  │  ├─lexer.rs
│  │  ├─parser.rs
│  │  ├─semantic.rs
│  │  └─main.rs
├─examples
│  ├─source
│  ├─asm
│  ├─ir
│  └─bytecode
├─Docs
│  ├─01_入口
│  ├─02_语言规范
│  ├─03_实现设计
│  └─99_历史资料
└─firmware
   ├─common
   ├─stm32f407
   └─esp32s3
```
