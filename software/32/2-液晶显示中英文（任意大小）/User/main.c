/* STM32F103 VM demo entry based on Fire example project. */

#include "main.h"

#include "vm_core.h"
#include "vm_hal.h"
#include "vm_program.h"
#include "bootloader_hbc.h"

#include "./led/bsp_led.h"
#include "./usart/bsp_debug_usart.h"
#include "./lcd/bsp_ili9341_lcd.h"

#include <stdio.h>

static uint16_t g_lcd_line = 0;
#define KEY1_PIN GPIO_PIN_0
#define KEY1_PORT GPIOA
#define KEY1_CLK_ENABLE() __HAL_RCC_GPIOA_CLK_ENABLE()
#define KEY2_PIN GPIO_PIN_13
#define KEY2_PORT GPIOC
#define KEY2_CLK_ENABLE() __HAL_RCC_GPIOC_CLK_ENABLE()

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
    GPIO_InitTypeDef gpio_init = {0};
    KEY1_CLK_ENABLE();
    KEY2_CLK_ENABLE();

    gpio_init.Mode = GPIO_MODE_INPUT;
    gpio_init.Pull = GPIO_PULLUP;
    gpio_init.Speed = GPIO_SPEED_FREQ_HIGH;

    gpio_init.Pin = KEY1_PIN;
    HAL_GPIO_Init(KEY1_PORT, &gpio_init);
    gpio_init.Pin = KEY2_PIN;
    HAL_GPIO_Init(KEY2_PORT, &gpio_init);
}

void hal_init(void) {
    LED_GPIO_Config();
    DEBUG_USART_Config();

    ILI9341_Init();
    ILI9341_GramScan(6);
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
    return HAL_GetTick();
}

void hal_delay_ms(uint32_t delay_ms) {
    HAL_Delay(delay_ms);
}

int hal_key1_read(void) {
    return HAL_GPIO_ReadPin(KEY1_PORT, KEY1_PIN) == GPIO_PIN_RESET ? 1 : 0;
}

int hal_key2_read(void) {
    return HAL_GPIO_ReadPin(KEY2_PORT, KEY2_PIN) == GPIO_PIN_RESET ? 1 : 0;
}

int main(void) {
    VmState vm;
    VmErrorCode error;
    BootUpdateResult update_result;
    int led_on = 0;
    const uint8_t *program_buffer = g_vm_program;
    uint32_t program_size = g_vm_program_size;

    HAL_Init();
    SystemClock_Config();
    hal_init();

    hal_log_int("vm_boot", 1);
    update_result = bootloader_try_uart_update(1500);
    hal_log_int("bl_update", (int)update_result);
    if (bootloader_load_program_from_flash(&program_buffer, &program_size) == 0) {
        hal_log_int("bl_slot", 1);
    } else {
        program_buffer = g_vm_program;
        program_size = g_vm_program_size;
        hal_log_int("bl_slot", 0);
    }

    error = vm_load_program(&vm, program_buffer, program_size);
    if (error != VM_OK) {
        hal_log_int("vm_load_err", (int)error);
        while (1) {
        }
    }

    hal_log_int("vm_loaded", 1);

    while (1) {
        error = vm_load_program(&vm, program_buffer, program_size);
        if (error != VM_OK) {
            hal_log_int("vm_load_err", (int)error);
            hal_delay_ms(1000);
            continue;
        }

        error = vm_run(&vm, 100000);
        if (error != VM_OK) {
            hal_log_int("vm_run_err", (int)error);
            hal_delay_ms(1000);
            continue;
        }

        hal_log_int("vm_retval", vm.retval);
        if (vm.retval != 0) {
            led_on = !led_on;
            hal_led_write(led_on);
            hal_log_int("led_on", led_on);
            hal_log_int("tick_ms", (int)hal_millis());
        }

        hal_delay_ms(100);
    }
}

void SystemClock_Config(void) {
    RCC_ClkInitTypeDef clkinitstruct = {0};
    RCC_OscInitTypeDef oscinitstruct = {0};

    oscinitstruct.OscillatorType = RCC_OSCILLATORTYPE_HSE;
    oscinitstruct.HSEState = RCC_HSE_ON;
    oscinitstruct.HSEPredivValue = RCC_HSE_PREDIV_DIV1;
    oscinitstruct.PLL.PLLState = RCC_PLL_ON;
    oscinitstruct.PLL.PLLSource = RCC_PLLSOURCE_HSE;
    oscinitstruct.PLL.PLLMUL = RCC_PLL_MUL9;
    if (HAL_RCC_OscConfig(&oscinitstruct) != HAL_OK) {
        while (1) {
        }
    }

    clkinitstruct.ClockType = RCC_CLOCKTYPE_SYSCLK | RCC_CLOCKTYPE_HCLK | RCC_CLOCKTYPE_PCLK1 | RCC_CLOCKTYPE_PCLK2;
    clkinitstruct.SYSCLKSource = RCC_SYSCLKSOURCE_PLLCLK;
    clkinitstruct.AHBCLKDivider = RCC_SYSCLK_DIV1;
    clkinitstruct.APB2CLKDivider = RCC_HCLK_DIV1;
    clkinitstruct.APB1CLKDivider = RCC_HCLK_DIV2;
    if (HAL_RCC_ClockConfig(&clkinitstruct, FLASH_LATENCY_2) != HAL_OK) {
        while (1) {
        }
    }
}
