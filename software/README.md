# 嵌入式简化编译器（Rust）

本项目实现了一个最小可运行编译器流程：

- 词法分析（Lexer）
- 语法分析（递归下降 Parser）
- 语义检查（声明与类型检查）
- IR 生成（TAC 三地址码）
- 后端汇编模板生成（`stm32f403` / `esp32`）

项目根目录：`E:\02_competition\software`

## 1. 目录说明

- `compiler/`：编译器源码（Rust）
- `examples/main.mc`：示例输入源码
- `examples/main_stm32.asm`：示例输出（STM32 风格）
- `examples/main_esp32.asm`：示例输出（ESP32 风格）
- `Docs/编译器设计文档.md`：设计文档

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
cd /d E:\02_competition\software\compiler
```

### 4.2 生成 STM32 风格汇编

```bat
cargo run -- ..\examples\main.mc -o ..\examples\main_stm32.asm --target stm32f403
```

### 4.3 生成 ESP32 风格汇编

```bat
cargo run -- ..\examples\main.mc -o ..\examples\main_esp32.asm --target esp32
```

### 4.4 查看输出结果

```bat
type E:\02_competition\software\examples\main_stm32.asm
type E:\02_competition\software\examples\main_esp32.asm
```

## 5. 最快复现命令（复制即用）

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d E:\02_competition\software\compiler
cargo run -- ..\examples\main.mc -o ..\examples\main_stm32.asm --target stm32f403
cargo run -- ..\examples\main.mc -o ..\examples\main_esp32.asm --target esp32
```

## 6. Demo 源码（当前）

`examples/main.mc` 内容是一个简单函数，包含：

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
cd /d E:\02_competition\software\compiler
cargo run -- ..\examples\stm32_blink.mc -o ..\examples\stm32_blink.asm --target stm32f403
```

### 9.2 生成 ESP32 点灯汇编

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d E:\02_competition\software\compiler
cargo run -- ..\examples\esp32_blink.mc -o ..\examples\esp32_blink.asm --target esp32
```

### 9.3 Demo 文件说明

- 输入源码：
  - `examples/stm32_blink.mc`
  - `examples/esp32_blink.mc`
- 输出汇编：
  - `examples/stm32_blink.asm`
  - `examples/esp32_blink.asm`
- GPIO 替换指引：
  - `examples/gpio_interface_todo.md`

> 当前 GPIO 是占位接口变量，目的是先跑通编译链路。你可以按开发板手册把占位变量替换成真实寄存器/HAL 调用。
> - 占位变量：`stm32_gpio_mode_reg` / `esp32_gpio_enable_reg`
> - 真实寄存器：`GPIOA_MODER` / `GPIO_ENABLE_REG`
> - 真实 HAL 调用：`HAL_GPIO_Init` / `HAL_GPIO_Init`

## 10. 文件结构
``` 
E:.
├─compiler
│  ├─src
│  │  ├─ast.rs
│  │  ├─backend.rs
│  │  ├─frontend.rs
│  │  ├─ir.rs
│  │  ├─lexer.rs
│  │  ├─parser.rs
│  │  ├─semantic.rs
│  │  └─main.rs
│  └─target
│      ├─debug
│      │      
│      └─flycheck0
├─Docs
│  └─design
└─examples
```