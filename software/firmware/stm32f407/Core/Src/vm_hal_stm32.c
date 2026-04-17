/* vm_hal_stm32.c -- STM32F407 平台 HAL 实现
 *
 * 基于 STM32 HAL 库实现 vm_hal.h 定义的接口。
 * 依赖：STM32F407-Discovery 板（PD12-PD15 LED, USART2）。
 */

#include "vm_hal.h"
#include "vm_core.h"

#include "stm32f4xx_hal.h"

/* USART2 用于串口日志输出 (PA2 TX) */
extern UART_HandleTypeDef huart2;

/* LED 引脚: PD12 (Green) */
#define LED_PORT  GPIOD
#define LED_PIN   GPIO_PIN_12

void hal_init(void) {
    /* GPIO 和 USART 已在 MX_GPIO_Init / MX_USART2_UART_Init 中完成，
     * 此处无需额外初始化。 */
}

void hal_led_write(int on) {
    if (on) {
        HAL_GPIO_WritePin(LED_PORT, LED_PIN, GPIO_PIN_SET);
    } else {
        HAL_GPIO_WritePin(LED_PORT, LED_PIN, GPIO_PIN_RESET);
    }
}

void hal_led_toggle(void) {
    HAL_GPIO_TogglePin(LED_PORT, LED_PIN);
}

void hal_log_int(const char *key, int value) {
    char buf[80];
    int len = snprintf(buf, sizeof(buf), "[VM] %s = %d\r\n", key, value);
    if (len > 0) {
        HAL_UART_Transmit(&huart2, (uint8_t *)buf, (uint16_t)len, HAL_MAX_DELAY);
    }
}

uint32_t hal_millis(void) {
    return (uint32_t)HAL_GetTick();
}

void hal_delay_ms(uint32_t delay_ms) {
    HAL_Delay(delay_ms);
}
