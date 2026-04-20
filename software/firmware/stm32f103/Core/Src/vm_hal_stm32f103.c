/* vm_hal_stm32f103.c -- STM32F103(指南者) 平台 HAL 实现
 *
 * 功能：
 * 1) LED 点灯：使用板载 RGB 中绿色通道（PB0）
 * 2) 串口日志：USART1 (PA9/PA10)
 * 3) 屏幕日志：ILI9341 LCD 显示 key=value
 *
 * 依赖硬件资料目录中的 BSP：
 * - User/led/bsp_led.h
 * - User/usart/bsp_usart.h
 * - User/lcd/bsp_ili9341_lcd.h
 */

#include "vm_hal.h"
#include "vm_core.h"

#include "stm32f10x.h"

#include "./led/bsp_led.h"
#include "./usart/bsp_usart.h"
#include "./lcd/bsp_ili9341_lcd.h"

#include <stdio.h>

static volatile uint32_t g_tick_ms = 0;
static uint16_t g_lcd_line = 0;
#define KEY1_PIN GPIO_Pin_0
#define KEY1_PORT GPIOA
#define KEY2_PIN GPIO_Pin_13
#define KEY2_PORT GPIOC

static void lcd_clear_line(uint16_t line_index) {
    uint16_t y = (uint16_t)(line_index * 16U);
    if (y >= LCD_Y_LENGTH) {
        return;
    }
    ILI9341_Clear(0, y, LCD_X_LENGTH, 16);
}

static void lcd_log_line(const char *text) {
    uint16_t y = (uint16_t)(g_lcd_line * 16U);
    if (y >= LCD_Y_LENGTH) {
        g_lcd_line = 0;
        y = 0;
        ILI9341_Clear(0, 0, LCD_X_LENGTH, LCD_Y_LENGTH);
    }

    lcd_clear_line(g_lcd_line);
    ILI9341_DispString_EN(0, y, (char *)text);

    g_lcd_line++;
}

static void key_gpio_init(void) {
    GPIO_InitTypeDef gpio_init;
    RCC_APB2PeriphClockCmd(RCC_APB2Periph_GPIOA | RCC_APB2Periph_GPIOC, ENABLE);

    gpio_init.GPIO_Mode = GPIO_Mode_IPU;
    gpio_init.GPIO_Speed = GPIO_Speed_50MHz;

    gpio_init.GPIO_Pin = KEY1_PIN;
    GPIO_Init(KEY1_PORT, &gpio_init);
    gpio_init.GPIO_Pin = KEY2_PIN;
    GPIO_Init(KEY2_PORT, &gpio_init);
}

void hal_init(void) {
    LED_GPIO_Config();
    USART_Config();

    ILI9341_Init();
    ILI9341_GramScan(3);
    LCD_SetBackColor(BLACK);
    LCD_SetTextColor(WHITE);
    ILI9341_Clear(0, 0, LCD_X_LENGTH, LCD_Y_LENGTH);
    key_gpio_init();

    LED2_OFF;
    lcd_log_line("[VM] f103 hal init");
}

void hal_led_write(int on) {
    if (on) {
        LED2_ON;
    } else {
        LED2_OFF;
    }
}

void hal_led_toggle(void) {
    LED2_TOGGLE;
}

void hal_log_int(const char *key, int value) {
    char buf[96];
    int len = snprintf(buf, sizeof(buf), "[VM] %s = %d", key, value);
    if (len <= 0) {
        return;
    }

    printf("%s\r\n", buf);
    lcd_log_line(buf);
}

uint32_t hal_millis(void) {
    return g_tick_ms;
}

void hal_delay_ms(uint32_t delay_ms) {
    uint32_t start = g_tick_ms;
    while ((g_tick_ms - start) < delay_ms) {
        /* busy wait */
    }
}

int hal_key1_read(void) {
    return GPIO_ReadInputDataBit(KEY1_PORT, KEY1_PIN) == Bit_RESET ? 1 : 0;
}

int hal_key2_read(void) {
    return GPIO_ReadInputDataBit(KEY2_PORT, KEY2_PIN) == Bit_RESET ? 1 : 0;
}

void SysTick_Handler(void) {
    g_tick_ms++;
}
