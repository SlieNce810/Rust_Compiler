# GPIO 接口替换说明（点灯 Demo）

这两个 Demo 先用“占位变量”表达 GPIO 行为，目的是先让编译链路跑通。
你后续只需要按开发板把占位变量替换成真实寄存器或 HAL 接口。

## 1. STM32 Demo

源码文件：`stm32_blink.mc`

占位变量：

- `stm32_gpio_mode_reg`：GPIO 模式初始化（输出模式）
- `stm32_gpio_output_reg`：LED 电平输出（0/1）

建议替换：

1. 初始化阶段：替换 `stm32_gpio_mode_reg = 1` 为真实 GPIO 初始化代码
2. 循环阶段：替换 `stm32_gpio_output_reg = led_state` 为真实写引脚电平代码

## 2. ESP32 Demo

源码文件：`esp32_blink.mc`

占位变量：

- `esp32_gpio_enable_reg`：GPIO 方向使能（输出）
- `esp32_gpio_output_reg`：LED 电平输出（0/1）

建议替换：

1. 初始化阶段：替换 `esp32_gpio_enable_reg = 1` 为真实 GPIO 方向配置
2. 循环阶段：替换 `esp32_gpio_output_reg = led_state` 为真实写引脚电平代码

## 3. 延时逻辑说明

当前延时是软件空转：

- `delay_counter = delay_limit`
- `while (delay_counter > 0) { delay_counter = delay_counter - 1; }`

你后续可替换成：

- STM32: SysTick / HAL_Delay
- ESP32: FreeRTOS delay 或定时器接口
