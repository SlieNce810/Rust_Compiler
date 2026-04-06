# 安全型 `.mc` 语言规范草案

本文档定义一版面向嵌入式的“安全型 `.mc`”语言规范。

目标不是把 `.mc` 直接做成完整 Rust，而是借鉴 Rust 的安全思想，做一门更适合端侧控制、资源受限、可静态检查的嵌入式语言。

这版规范重点覆盖四件事：

1. 资源类型
2. 安全外设接口
3. 状态检查规则
4. 编译期错误规则

---

## 1. 设计目标

安全型 `.mc`` 的目标不是“表达一切”，而是“只允许安全地表达常见嵌入式控制逻辑”。

### 1.1 核心目标

- 禁止裸指针和任意地址访问
- 禁止未初始化资源使用
- 禁止外设状态不合法时调用接口
- 禁止同一硬件资源被多个逻辑块无约束复用
- 尽量把错误提前到编译期

### 1.2 适合的场景

- 点灯
- 按键轮询
- 传感器轮询
- UART 日志输出
- PWM 占空比控制
- ADC 采样
- 简单状态机
- 任务主循环

### 1.3 暂不追求的能力

- 完整 borrow checker
- 任意指针运算
- 自定义内存分配
- 动态装载复杂库
- 任意中断回调图灵完备表达

这不是退让，而是为了让语言先在嵌入式控制领域站稳。

---

## 2. 语言分层

安全型 `.mc` 建议分三层。

### 2.1 基础值类型

用于表达普通计算和条件判断：

- `int`
- `float`
- `bool`

### 2.2 资源类型

用于表达“硬件资源的所有权和用途”：

- `gpio`
- `uart`
- `adc`
- `timer`
- `pwm`
- `spi`
- `i2c`

### 2.3 配置类型

用于表达初始化时需要的参数：

- `pin`
- `baud`
- `channel`
- `frequency`
- `duty`
- `sample_rate`

这些类型的核心价值不是“语法好看”，而是让编译器知道你在配置什么硬件。

---

## 3. 资源类型规范

资源类型不是普通变量。它们表达的是“硬件对象”。

### 3.1 GPIO

```mc
gpio led_pin;
```

含义：

- `led_pin` 是一个 GPIO 资源句柄
- 它不能像 `int` 一样做加减乘除

### 3.2 UART

```mc
uart debug_uart;
```

含义：

- `debug_uart` 是一个串口资源
- 它只能调用串口相关安全接口

### 3.3 ADC

```mc
adc temp_sensor;
```

含义：

- `temp_sensor` 是 ADC 输入通道资源
- 只能做 ADC 初始化、启动、读取

### 3.4 Timer

```mc
timer blink_timer;
```

含义：

- `blink_timer` 是一个定时器资源
- 只能做定时器配置、启动、停止、查询

### 3.5 PWM

```mc
pwm fan_pwm;
```

含义：

- `fan_pwm` 是 PWM 输出资源
- 只能做频率、占空比和启停控制

---

## 4. 资源声明原则

资源必须显式声明，不能隐式创建。

### 4.1 显式声明

```mc
gpio led_pin;
uart debug_uart;
```

### 4.2 禁止把资源当普通值用

错误示例：

```mc
gpio led_pin;
int a;

a = led_pin;
```

原因：

- `gpio` 不是整数
- 资源句柄不能参与普通值运算

### 4.3 禁止资源复制

错误示例：

```mc
gpio led_a;
gpio led_b;

led_b = led_a;
```

原因：

- 这会制造“两个名字指向同一硬件资源”的歧义
- 安全型 `.mc` 应默认资源不可复制

这条规则是对 Rust 所有权思想的简化借鉴。

---

## 5. 资源状态模型

每个资源都有状态。

编译器不仅检查类型，还要检查“当前状态下是否允许调用这个接口”。

### 5.1 GPIO 状态

建议状态集合：

- `Declared`
- `Bound`
- `ConfiguredInput`
- `ConfiguredOutput`
- `High`
- `Low`

简化理解：

1. 先声明
2. 再绑定引脚
3. 再配置方向
4. 然后才能读写

### 5.2 UART 状态

建议状态集合：

- `Declared`
- `Bound`
- `Configured`
- `Ready`

### 5.3 ADC 状态

建议状态集合：

- `Declared`
- `Bound`
- `Configured`
- `Ready`

### 5.4 Timer 状态

建议状态集合：

- `Declared`
- `Configured`
- `Running`
- `Stopped`

### 5.5 PWM 状态

建议状态集合：

- `Declared`
- `Bound`
- `Configured`
- `Enabled`
- `Disabled`

---

## 6. 安全外设接口规范

外设访问必须通过内建安全接口完成。

禁止用户直接写寄存器地址。

---

## 7. GPIO 安全接口

### 7.1 绑定引脚

```mc
gpio_bind(led_pin, pin("PA5"));
gpio_bind(led_pin, pin("GPIO2"));
```

含义：

- 将逻辑资源绑定到物理引脚

编译器检查：

- 资源必须是 `gpio`
- 资源尚未绑定
- 引脚名称格式合法

### 7.2 配置为输出

```mc
gpio_set_output(led_pin);
```

编译器检查：

- `led_pin` 必须已经绑定
- 当前不能已经处于输出状态冲突模式

### 7.3 配置为输入

```mc
gpio_set_input(button_pin);
```

### 7.4 写高电平

```mc
gpio_write_high(led_pin);
```

编译器检查：

- 资源必须是 GPIO
- 必须已配置为输出

### 7.5 写低电平

```mc
gpio_write_low(led_pin);
```

### 7.6 切换电平

```mc
gpio_toggle(led_pin);
```

### 7.7 读取输入

```mc
int key_value;
key_value = gpio_read(button_pin);
```

编译器检查：

- 必须已配置为输入

---

## 8. UART 安全接口

### 8.1 绑定串口

```mc
uart_bind(debug_uart, channel("UART1"));
```

### 8.2 配置波特率

```mc
uart_set_baud(debug_uart, baud(115200));
```

### 8.3 启用串口

```mc
uart_enable(debug_uart);
```

### 8.4 发送整数

```mc
uart_write_int(debug_uart, sensor_value);
```

### 8.5 发送布尔值

```mc
uart_write_bool(debug_uart, is_ready);
```

编译器检查：

- UART 必须已配置并启用
- 参数类型必须匹配

---

## 9. ADC 安全接口

### 9.1 绑定 ADC 通道

```mc
adc_bind(temp_sensor, channel("ADC1_CH3"));
```

### 9.2 配置采样

```mc
adc_set_sample_rate(temp_sensor, sample_rate(1000));
```

### 9.3 启用 ADC

```mc
adc_enable(temp_sensor);
```

### 9.4 读取采样值

```mc
int temp_value;
temp_value = adc_read(temp_sensor);
```

编译器检查：

- ADC 必须已启用
- 返回值必须接到兼容类型变量

---

## 10. Timer 安全接口

### 10.1 配置周期

```mc
timer_set_period(blink_timer, frequency(2));
```

### 10.2 启动定时器

```mc
timer_start(blink_timer);
```

### 10.3 停止定时器

```mc
timer_stop(blink_timer);
```

### 10.4 查询是否到期

```mc
bool has_tick;
has_tick = timer_is_ready(blink_timer);
```

这比空转延时更适合真实嵌入式场景。

---

## 11. PWM 安全接口

### 11.1 绑定输出

```mc
pwm_bind(fan_pwm, pin("PA8"));
```

### 11.2 设置频率

```mc
pwm_set_frequency(fan_pwm, frequency(20000));
```

### 11.3 设置占空比

```mc
pwm_set_duty(fan_pwm, duty(50));
```

### 11.4 启用 PWM

```mc
pwm_enable(fan_pwm);
```

编译器检查：

- 占空比范围合法
- PWM 已绑定并配置

---

## 12. 状态检查规则

这是安全型 `.mc` 的核心。

编译器不仅检查“类型对不对”，还检查“这个资源在当前阶段能不能这么用”。

### 12.1 规则一：未绑定不能配置

错误示例：

```mc
gpio led_pin;
gpio_set_output(led_pin);
```

错误原因：

- 资源还没有绑定到具体引脚

### 12.2 规则二：未配置不能使用

错误示例：

```mc
gpio led_pin;
gpio_bind(led_pin, pin("PA5"));
gpio_write_high(led_pin);
```

错误原因：

- 已绑定，但未设为输出

### 12.3 规则三：输入资源不能写

错误示例：

```mc
gpio button_pin;
gpio_bind(button_pin, pin("PA0"));
gpio_set_input(button_pin);
gpio_write_high(button_pin);
```

### 12.4 规则四：输出资源不能读输入值

错误示例：

```mc
gpio led_pin;
gpio_bind(led_pin, pin("PA5"));
gpio_set_output(led_pin);
int value;
value = gpio_read(led_pin);
```

### 12.5 规则五：未启用的外设不能工作

错误示例：

```mc
uart debug_uart;
uart_bind(debug_uart, channel("UART1"));
uart_write_int(debug_uart, 123);
```

### 12.6 规则六：资源不可重复绑定

错误示例：

```mc
gpio led_pin;
gpio_bind(led_pin, pin("PA5"));
gpio_bind(led_pin, pin("PB3"));
```

### 12.7 规则七：两个资源不能绑定同一独占硬件

错误示例：

```mc
gpio led_a;
gpio led_b;

gpio_bind(led_a, pin("PA5"));
gpio_bind(led_b, pin("PA5"));
```

除非语言明确支持共享，否则默认禁止。

### 12.8 规则八：配置值必须在安全范围内

错误示例：

```mc
pwm_set_duty(fan_pwm, duty(150));
```

错误原因：

- 占空比超出 `0..100`

---

## 13. 编译期错误规则

下面是建议必须支持的编译期错误类别。

### 13.1 类型错误

例如：

- `gpio` 赋给 `int`
- `bool` 和 `gpio` 比较
- `uart_write_int` 传入 `gpio`

错误示例：

```mc
gpio led_pin;
int a;
a = led_pin;
```

### 13.2 未声明错误

```mc
gpio_set_output(led_pin);
```

### 13.3 未绑定错误

```mc
gpio led_pin;
gpio_set_output(led_pin);
```

### 13.4 未配置错误

```mc
gpio led_pin;
gpio_bind(led_pin, pin("PA5"));
gpio_write_high(led_pin);
```

### 13.5 状态冲突错误

```mc
gpio_set_input(led_pin);
gpio_set_output(led_pin);
```

如果语言不允许重复覆盖配置，就应报错。

### 13.6 资源冲突错误

```mc
gpio_bind(led_a, pin("PA5"));
gpio_bind(led_b, pin("PA5"));
```

### 13.7 参数范围错误

```mc
uart_set_baud(debug_uart, baud(-1));
pwm_set_duty(fan_pwm, duty(200));
```

### 13.8 平台不支持错误

```mc
gpio_bind(led_pin, pin("PA5"));
```

如果目标平台是 ESP32，而 `PA5` 这种命名不合法，编译器应报平台相关错误。

### 13.9 返回值错误

```mc
func int main() {
    return true;
}
```

### 13.10 未初始化值使用错误

```mc
int delay_limit;
int delay_counter;

delay_counter = delay_limit;
```

如果 `delay_limit` 在使用前未赋值，应报错。

这条规则非常重要。它是最容易落地、收益也非常高的一条安全规则。

---

## 14. 建议的错误信息风格

错误信息不要只写“syntax error”。

应当包含：

1. 错误类型
2. 资源名或变量名
3. 当前状态
4. 期望状态
5. 修复建议

示例：

```text
error: gpio_write_high requires output gpio
resource: led_pin
current state: Bound
expected state: ConfiguredOutput
help: call gpio_set_output(led_pin) before writing
```

这样的错误信息非常适合端侧控制语言。

---

## 15. 示例：安全型点灯程序

下面是一版建议中的安全型 `.mc` 点灯写法。

```mc
func int main() {
    bool is_running;
    gpio led_pin;
    timer blink_timer;

    is_running = true;

    gpio_bind(led_pin, pin("PA5"));
    gpio_set_output(led_pin);

    timer_set_period(blink_timer, frequency(2));
    timer_start(blink_timer);

    while (is_running) {
        if (timer_is_ready(blink_timer)) {
            gpio_toggle(led_pin);
        }
    }

    return 0;
}
```

这比当前占位变量版更安全，原因是：

- `led_pin` 是明确的 GPIO 资源，不是普通 `int`
- 先绑定，再配置，再使用
- 用定时器表达“闪烁节拍”，而不是空转延时

---

## 16. 示例：安全型串口日志程序

```mc
func int main() {
    bool is_running;
    uart debug_uart;
    int sensor_value;

    is_running = true;
    sensor_value = 0;

    uart_bind(debug_uart, channel("UART1"));
    uart_set_baud(debug_uart, baud(115200));
    uart_enable(debug_uart);

    while (is_running) {
        sensor_value = sensor_value + 1;
        uart_write_int(debug_uart, sensor_value);
    }

    return 0;
}
```

---

## 17. 实现优先级建议

如果你要把这版规范落到现有编译器，优先级建议如下。

### 第一阶段

- 增加资源类型：`gpio`、`uart`、`timer`
- 增加内建接口语法
- 增加未初始化变量检查
- 增加资源状态检查

### 第二阶段

- 增加 `adc`、`pwm`
- 增加参数范围检查
- 增加平台引脚合法性检查

### 第三阶段

- 增加资源冲突分析
- 增加更细的状态流分析
- 增加端侧可执行 IR / bytecode 映射

这条路线比直接做“Rust 完整所有权系统”更稳，也更适合你现在的工程阶段。

---

## 18. 结论

安全型 `.mc` 不需要复制 Rust 的全部复杂性，但应该明确继承下面这些思想：

- 资源必须显式声明
- 资源不能随意复制
- 状态不合法时禁止操作
- 非法硬件访问尽量编译期报错
- 外设访问必须走受控接口

如果你按这套规范推进，`.mc` 会逐步从“教学用小语言”升级成“安全的嵌入式控制 DSL”。
