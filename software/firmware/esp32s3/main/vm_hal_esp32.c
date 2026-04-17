/* vm_hal_esp32.c -- ESP32-S3 平台 HAL 实现
 *
 * 基于 ESP-IDF 实现 vm_hal.h 定义的接口。
 * 依赖：ESP32-S3-DevKitC（GPIO2 LED, UART0 USB-JTAG）。
 */

#include "vm_hal.h"
#include "vm_core.h"

#include "driver/gpio.h"
#include "driver/uart.h"
#include "esp_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

/* LED 引脚: GPIO2 (ESP32-S3-DevKitC 内置蓝灯) */
#define BLINK_GPIO 2

static const char *TAG = "VM";

void hal_init(void) {
    /* GPIO 初始化 */
    gpio_reset_pin(BLINK_GPIO);
    gpio_set_direction(BLINK_GPIO, GPIO_MODE_OUTPUT);

    /* UART 已由 ESP-IDF 自动初始化 (USB-JTAG / UART0) */
}

void hal_led_write(int on) {
    gpio_set_level(BLINK_GPIO, on ? 1 : 0);
}

void hal_led_toggle(void) {
    static int level = 0;
    level = !level;
    gpio_set_level(BLINK_GPIO, level);
}

void hal_log_int(const char *key, int value) {
    ESP_LOGI(TAG, "%s = %d", key, value);
}

uint32_t hal_millis(void) {
    return (uint32_t)(xTaskGetTickCount() * portTICK_PERIOD_MS);
}

void hal_delay_ms(uint32_t delay_ms) {
    vTaskDelay(pdMS_TO_TICKS(delay_ms));
}
