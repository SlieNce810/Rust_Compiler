#ifndef BOOTLOADER_HBC_ESP32_H
#define BOOTLOADER_HBC_ESP32_H

#include <stdint.h>

typedef enum EspBootUpdateResult {
    ESP_BOOT_UPDATE_IDLE = 0,
    ESP_BOOT_UPDATE_TIMEOUT = 1,
    ESP_BOOT_UPDATE_BAD_HEADER = 2,
    ESP_BOOT_UPDATE_BAD_LENGTH = 3,
    ESP_BOOT_UPDATE_BAD_CRC = 4,
    ESP_BOOT_UPDATE_FLASH_ERROR = 5,
    ESP_BOOT_UPDATE_NO_PARTITION = 6,
    ESP_BOOT_UPDATE_OK = 7
} EspBootUpdateResult;

EspBootUpdateResult esp_bootloader_try_uart_update(uint32_t window_ms);
int esp_bootloader_load_program_from_flash(const uint8_t **buffer, uint32_t *size);

#endif
