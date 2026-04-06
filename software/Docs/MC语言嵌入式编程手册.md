# `.mc` 语言嵌入式编程手册

本文档不是介绍编译器内部，而是教你如何直接使用 `.mc` 写程序。

重点包括：

1. `.mc` 能写什么
2. `.mc` 目前不能写什么
3. 如何用 `.mc` 写点灯
4. 如何扩展到更多嵌入式控制逻辑
5. 如何把“占位接口”换成你开发板上的真实接口

---

## 1. 先建立一个正确认识

当前 `.mc` 是你这套编译器的**源语言**。

你写的是：

- `xxx.mc`

编译器会把它变成：

1. 词法 token
2. 语法树 AST
3. 三地址码 IR
4. `stm32` 或 `esp32` 风格汇编

所以你平时真正手写的是 `.mc`，不是中间语言，也不是汇编。

---

## 2. `.mc` 当前支持什么

当前语言子集支持：

- 类型：`int`、`float`、`bool`
- 函数：`func int main() { ... }`
- 变量声明
- 变量赋值
- `if / else`
- `while`
- `return`
- 表达式：
  - `+`
  - `-`
  - `*`
  - `/`
  - `<`
  - `>`
  - `==`

你可以把它理解成一个“极简版类 C 语言”。

---

## 3. `.mc` 当前不支持什么

当前还不支持：

- 数组
- 结构体
- 指针
- 外部函数调用
- 真实寄存器地址语法
- 中断
- 定时器外设语法
- 严格板级 HAL 封装

这意味着：

你现在最适合用 `.mc` 写的是**控制逻辑**，例如：

- 点灯
- 状态切换
- 延时轮询
- 阈值判断
- 简单传感器轮询框架
- 简单任务主循环

---

## 4. `.mc` 最基本的程序长什么样

一个最小程序示例：

```c
func int main() {
    int a;
    int b;

    a = 1;
    b = 2;

    return a + b;
}
```

你可以先记住这几个硬规则：

1. 程序入口目前写成 `main`
2. 每个变量都要先声明再使用
3. 每条语句后面都要有 `;`
4. `if` 和 `while` 的条件要放在 `()`
5. 代码块要放在 `{ }`
6. 最后要 `return`

---

## 5. 如何理解“用 `.mc` 写点灯”

### 5.1 点灯的本质

点灯不是“神秘硬件操作”，本质上就是三步：

1. 把某个 GPIO 配成输出
2. 不断改变这个 GPIO 的输出电平
3. 每次改变之后等一段时间

翻译成程序逻辑就是：

1. 初始化
2. 进入死循环
3. 在循环里反复执行：
   - LED 取反
   - 写电平
   - 延时

---

## 6. STM32 点灯写法示例

当前项目中的示例文件：

- [stm32_blink.mc](/E:/02_competition/software/examples/stm32_blink.mc)

源码如下：

```c
func int main() {
    bool is_running;
    int led_state;
    int delay_counter;
    int delay_limit;
    int stm32_gpio_mode_reg;
    int stm32_gpio_output_reg;

    is_running = true;
    led_state = 0;
    delay_limit = 5000;

    stm32_gpio_mode_reg = 1;

    while (is_running) {
        if (led_state == 0) {
            led_state = 1;
        } else {
            led_state = 0;
        }

        stm32_gpio_output_reg = led_state;

        delay_counter = delay_limit;
        while (delay_counter > 0) {
            delay_counter = delay_counter - 1;
        }
    }

    return 0;
}
```

### 6.1 逐段解释

#### 第一段：声明变量

```c
bool is_running;
int led_state;
int delay_counter;
int delay_limit;
int stm32_gpio_mode_reg;
int stm32_gpio_output_reg;
```

这些变量的意义是：

- `is_running`：主循环是否继续
- `led_state`：LED 当前状态，`0` 表示灭，`1` 表示亮
- `delay_counter`：延时倒计数
- `delay_limit`：每次延时的长度
- `stm32_gpio_mode_reg`：GPIO 模式寄存器占位变量
- `stm32_gpio_output_reg`：GPIO 输出寄存器占位变量

这里最关键的是最后两个变量。

它们现在还不是“真实 STM32 寄存器”，只是先把程序逻辑表达出来。这样做的目的是先让编译器链路跑通。

#### 第二段：初始化

```c
is_running = true;
led_state = 0;
delay_limit = 5000;

stm32_gpio_mode_reg = 1;
```

这段在做三件事：

1. 启动主循环
2. 让 LED 初始为灭
3. 配置 GPIO 为输出模式

其中：

```c
stm32_gpio_mode_reg = 1;
```

表示“这里本来应该写 STM32 的 GPIO 初始化代码”，现在先用占位变量代替。

#### 第三段：主循环

```c
while (is_running) {
    ...
}
```

这是嵌入式里最常见的结构。

意思是：

只要系统还在运行，就一直重复执行下面的控制逻辑。

#### 第四段：翻转 LED 状态

```c
if (led_state == 0) {
    led_state = 1;
} else {
    led_state = 0;
}
```

这段的作用非常直接：

- 如果当前是灭，就改成亮
- 否则改成灭

这就是“闪烁”的核心。

#### 第五段：把状态写到 GPIO

```c
stm32_gpio_output_reg = led_state;
```

这句表示：

把当前 LED 状态写入 GPIO 输出寄存器。

后续当你知道自己板子的真实接口后，这一行要换成真正的 GPIO 输出动作。

#### 第六段：软件延时

```c
delay_counter = delay_limit;
while (delay_counter > 0) {
    delay_counter = delay_counter - 1;
}
```

这段的目的是：

防止 LED 变化太快。

如果没有延时，LED 会在极短时间内变化，你肉眼看不到闪烁过程。

这里用的是**空转延时**，优点是实现简单，适合当前语言能力。

缺点是：

- 不精确
- 占 CPU

后续可以替换为：

- `SysTick`
- `HAL_Delay`
- `FreeRTOS delay`

---

## 7. ESP32 点灯写法示例

当前项目中的示例文件：

- [esp32_blink.mc](/E:/02_competition/software/examples/esp32_blink.mc)

和 STM32 版本几乎一样，只是接口变量名字换了：

```c
int esp32_gpio_enable_reg;
int esp32_gpio_output_reg;
```

它们的含义是：

- `esp32_gpio_enable_reg`：GPIO 输出使能占位变量
- `esp32_gpio_output_reg`：GPIO 输出电平占位变量

这表示 ESP32 版本的思路也是一样的：

1. 先让 GPIO 变成输出
2. 然后不断写 `0 / 1`

---

## 8. 如何编译 `.mc`

先进入环境：

```bat
"C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d E:\02_competition\software\compiler
```

### 编译 STM32 点灯

```bat
cargo run -- ..\examples\stm32_blink.mc -o ..\examples\stm32_blink.asm --target stm32f403
```

### 编译 ESP32 点灯

```bat
cargo run -- ..\examples\esp32_blink.mc -o ..\examples\esp32_blink.asm --target esp32
```

输出文件会放在：

- [stm32_blink.asm](/E:/02_competition/software/examples/stm32_blink.asm)
- [esp32_blink.asm](/E:/02_competition/software/examples/esp32_blink.asm)

---

## 9. `.mc` 适合怎么写嵌入式程序

当前阶段，建议你按下面这种思路写。

### 9.1 写成“初始化 + 主循环”

这是最适合 `.mc` 的结构：

```c
func int main() {
    bool is_running;

    is_running = true;

    // 初始化

    while (is_running) {
        // 主逻辑
    }

    return 0;
}
```

这类结构适合：

- 点灯
- 轮询按键
- 轮询传感器
- 简单控制回路

### 9.2 用变量表示硬件状态

例如：

```c
int led_state;
int motor_state;
int sensor_value;
int uart_send_reg;
```

这样写的好处是：

即使语言暂时不支持真实寄存器地址，你仍然能先把“控制逻辑”写清楚。

### 9.3 用 `if` 表示控制策略

例如温度阈值控制：

```c
if (sensor_value > 50) {
    motor_state = 1;
} else {
    motor_state = 0;
}
```

这类写法非常适合 `.mc` 当前能力。

### 9.4 用 `while` 表示持续轮询

例如：

```c
while (is_running) {
    sensor_value = 1;

    if (sensor_value > 0) {
        led_state = 1;
    } else {
        led_state = 0;
    }
}
```

这就是典型的轮询式嵌入式逻辑。

---

## 10. 除了点灯，还能写什么

按照当前语言能力，你已经可以写这些模式。

### 10.1 按键控制 LED

```c
func int main() {
    bool is_running;
    int key_value;
    int led_state;

    is_running = true;

    while (is_running) {
        key_value = 0;

        if (key_value == 0) {
            led_state = 1;
        } else {
            led_state = 0;
        }
    }

    return 0;
}
```

这里的 `key_value` 也可以先看成占位输入变量。

### 10.2 阈值控制风扇

```c
func int main() {
    bool is_running;
    int temperature_value;
    int fan_output_reg;

    is_running = true;

    while (is_running) {
        temperature_value = 60;

        if (temperature_value > 50) {
            fan_output_reg = 1;
        } else {
            fan_output_reg = 0;
        }
    }

    return 0;
}
```

### 10.3 固定节拍任务

```c
func int main() {
    bool is_running;
    int delay_counter;
    int delay_limit;
    int task_flag;

    is_running = true;
    delay_limit = 1000;

    while (is_running) {
        task_flag = 1;

        delay_counter = delay_limit;
        while (delay_counter > 0) {
            delay_counter = delay_counter - 1;
        }
    }

    return 0;
}
```

---

## 11. 你以后如何替换真实 GPIO 接口

当前推荐分三步替换。

### 第一步：保留逻辑，先只换“初始化”

例如把：

```c
stm32_gpio_mode_reg = 1;
```

替换成：

- STM32 板子的 GPIO 输出初始化逻辑

### 第二步：再换“输出”

例如把：

```c
stm32_gpio_output_reg = led_state;
```

替换成：

- 真正的 GPIO 写高低电平逻辑

### 第三步：最后替换“延时”

例如把软件空转：

```c
delay_counter = delay_limit;
while (delay_counter > 0) {
    delay_counter = delay_counter - 1;
}
```

替换成：

- 定时器
- 板级 delay
- RTOS 延时

为什么按这个顺序？

因为这样最稳：

1. 先保证 GPIO 初始化正确
2. 再保证输出行为正确
3. 最后再处理时间精度问题

---

## 12. 写 `.mc` 的实用建议

### 建议 1：先把“逻辑”写对，再考虑“硬件接口”

先写：

- 状态变量
- 条件逻辑
- 循环逻辑

等逻辑跑顺了，再把占位变量换成真实接口。

### 建议 2：一个变量只表达一个硬件含义

例如：

- `led_state`
- `gpio_output_reg`
- `motor_enable_reg`
- `sensor_value`

不要一个变量同时表达多个意思。

### 建议 3：每个主循环只做少数几件事

例如：

1. 读输入
2. 算状态
3. 写输出
4. 延时

这样后续无论看 `.mc`、IR 还是 `.asm` 都更容易追踪。

---

## 13. 当前最适合你的学习路径

你现在最合理的练习顺序是：

1. 先看懂 `stm32_blink.mc`
2. 把点灯改成“亮两次再灭两次”
3. 再写“按键控制点灯”
4. 再写“阈值控制输出”
5. 最后把占位接口替换成你板子的真实 GPIO

这条路径是渐进的，不会一下跳到过难的寄存器细节。

---

## 14. 一句话总结

`.mc` 当前最适合做的事情，不是直接替代完整 STM32/ESP-IDF 开发框架，而是先把**嵌入式控制逻辑**用一种简单、可编译、可落到汇编的形式写出来。

你可以先用它写：

- 点灯
- 轮询
- 条件控制
- 状态机雏形

再逐步把占位硬件变量替换成真实板级接口。
