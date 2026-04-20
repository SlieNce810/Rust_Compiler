#ifndef BOOTLOADER_HBC_H
#define BOOTLOADER_HBC_H

#include <stdint.h>

typedef enum BootUpdateResult {
    BOOT_UPDATE_IDLE = 0,
    BOOT_UPDATE_TIMEOUT = 1,
    BOOT_UPDATE_BAD_HEADER = 2,
    BOOT_UPDATE_BAD_LENGTH = 3,
    BOOT_UPDATE_BAD_CRC = 4,
    BOOT_UPDATE_FLASH_ERROR = 5,
    BOOT_UPDATE_OK = 6
} BootUpdateResult;

/* Wait UART packet in a short boot window and persist HBC payload to Flash. */
BootUpdateResult bootloader_try_uart_update(uint32_t window_ms);

/* Load validated HBC payload from Flash storage region. Returns 0 on success. */
int bootloader_load_program_from_flash(const uint8_t **buffer, uint32_t *size);

#endif
