/* main.c -- STM32F407 端侧解释器 Demo 主程序
 *
 * 流程：
 * 1. HAL 初始化（SystemClock / GPIO / USART2）
 * 2. 加载内置 blink.hbc 字节码
 * 3. 循环执行 VM：每次执行完毕后根据 retval 切换 LED
 * 4. 串口输出启动日志、状态切换、异常码
 *
 * 本文件提供最简的 STM32F407-Discovery 初始化代码，
 * 不依赖 STM32CubeMX 生成框架，可直接编译。
 */

#include "vm_core.h"
#include "vm_hal.h"
#include "vm_program.h"

#include "stm32f4xx_hal.h"

/* ---- 全局句柄 ---- */
UART_HandleTypeDef huart2;

/* ---- 私有函数声明 ---- */
void SystemClock_Config(void);
static void MX_GPIO_Init(void);
static void MX_USART2_UART_Init(void);

/* ---- 入口 ---- */
int main(void) {
    VmState vm;
    VmErrorCode error;
    int led_on = 0;

    /* 1. HAL 初始化 */
    HAL_Init();
    SystemClock_Config();
    MX_GPIO_Init();
    MX_USART2_UART_Init();
    hal_init();

    hal_log_int("vm_boot", 1);

    /* 2. 加载内置字节码 */
    error = vm_load_program(&vm, g_vm_program, g_vm_program_size);
    if (error != VM_OK) {
        hal_log_int("vm_load_err", (int)error);
        while (1) { /* 卡死，LED 不亮 */ }
    }

    hal_log_int("vm_loaded", 1);

    /* 3. 主循环：反复执行 VM，根据 retval 控制 LED */
    while (1) {
        /* 重新加载程序（重置 PC 和寄存器） */
        error = vm_load_program(&vm, g_vm_program, g_vm_program_size);
        if (error != VM_OK) {
            hal_log_int("vm_load_err", (int)error);
            hal_delay_ms(2000);
            continue;
        }

        /* 执行，步数限制防止死循环 */
        error = vm_run(&vm, 100000);

        if (error != VM_OK) {
            hal_log_int("vm_run_err", (int)error);
            hal_delay_ms(2000);
            continue;
        }

        hal_log_int("vm_retval", vm.retval);

        /* 根据 retval 切换 LED */
        if (vm.retval != 0) {
            led_on = !led_on;
            hal_led_write(led_on);
            hal_log_int("led_on", led_on);
            hal_log_int("tick_ms", (int)hal_millis());
        }

        /* 每轮之间短暂延时 */
        hal_delay_ms(100);
    }
}

/* ---- 系统时钟配置 (STM32F407-Discovery: HSE=8MHz, SYSCLK=168MHz) ---- */
void SystemClock_Config(void) {
    RCC_OscInitTypeDef osc = {0};
    RCC_ClkInitTypeDef clk = {0};

    osc.OscillatorType = RCC_OSCILLATORTYPE_HSE;
    osc.HSEState = RCC_HSE_ON;
    osc.PLL.PLLState = RCC_PLL_ON;
    osc.PLL.PLLSource = RCC_PLLSOURCE_HSE;
    osc.PLL.PLLM = 8;
    osc.PLL.PLLN = 336;
    osc.PLL.PLLP = RCC_PLLP_DIV2;
    osc.PLL.PLLQ = 7;
    HAL_RCC_OscConfig(&osc);

    clk.ClockType = RCC_CLOCKTYPE_HCLK | RCC_CLOCKTYPE_SYSCLK |
                    RCC_CLOCKTYPE_PCLK1 | RCC_CLOCKTYPE_PCLK2;
    clk.SYSCLKSource = RCC_SYSCLKSOURCE_PLLCLK;
    clk.AHBCLKDivider = RCC_SYSCLK_DIV1;
    clk.APB1CLKDivider = RCC_HCLK_DIV4;
    clk.APB2CLKDivider = RCC_HCLK_DIV2;
    HAL_RCC_ClockConfig(&clk, FLASH_LATENCY_5);
}

/* ---- USART2 初始化 (PA2=TX, PA3=RX, 115200 8N1) ---- */
static void MX_USART2_UART_Init(void) {
    huart2.Instance = USART2;
    huart2.Init.BaudRate = 115200;
    huart2.Init.WordLength = UART_WORDLENGTH_8B;
    huart2.Init.StopBits = UART_STOPBITS_1;
    huart2.Init.Parity = UART_PARITY_NONE;
    huart2.Init.Mode = UART_MODE_TX_RX;
    huart2.Init.HwFlowCtl = UART_HWCONTROL_NONE;
    huart2.Init.OverSampling = UART_OVERSAMPLING_16;
    HAL_UART_Init(&huart2);
}

/* ---- GPIO 初始化 ---- */
static void MX_GPIO_Init(void) {
    GPIO_InitTypeDef gpio = {0};

    __HAL_RCC_GPIOD_CLK_ENABLE();
    __HAL_RCC_GPIOA_CLK_ENABLE();

    /* LED: PD12 (Green) */
    gpio.Pin = GPIO_PIN_12;
    gpio.Mode = GPIO_MODE_OUTPUT_PP;
    gpio.Pull = GPIO_NOPULL;
    gpio.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOD, &gpio);

    /* USART2 TX: PA2 */
    gpio.Pin = GPIO_PIN_2;
    gpio.Mode = GPIO_MODE_AF_PP;
    gpio.Pull = GPIO_NOPULL;
    gpio.Speed = GPIO_SPEED_FREQ_VERY_HIGH;
    gpio.Alternate = GPIO_AF7_USART2;
    HAL_GPIO_Init(GPIOA, &gpio);

    /* USART2 RX: PA3 */
    gpio.Pin = GPIO_PIN_3;
    gpio.Mode = GPIO_MODE_AF_PP;
    gpio.Pull = GPIO_PULLUP;
    gpio.Speed = GPIO_SPEED_FREQ_VERY_HIGH;
    gpio.Alternate = GPIO_AF7_USART2;
    HAL_GPIO_Init(GPIOA, &gpio);
}

/* ---- HAL 回调 ---- */
void HAL_UART_MspInit(UART_HandleTypeDef *huart) {
    if (huart->Instance == USART2) {
        __HAL_RCC_USART2_CLK_ENABLE();
    }
}

void SysTick_Handler(void) {
    HAL_IncTick();
}
