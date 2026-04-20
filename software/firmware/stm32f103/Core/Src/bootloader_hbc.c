#include "bootloader_hbc.h"

#include "main.h"
#include "usart/bsp_debug_usart.h"

#include <string.h>

#define BOOT_PACKET_MAGIC_0 'H'
#define BOOT_PACKET_MAGIC_1 'U'
#define BOOT_PACKET_MAGIC_2 'P'
#define BOOT_PACKET_MAGIC_3 '1'
#define BOOT_PACKET_VERSION 1U

#define BOOT_RX_MAX_BYTES 61440U
#define BOOT_FLASH_PAGE_SIZE 2048U
#define BOOT_FLASH_DATA_ADDR 0x08070000U
#define BOOT_FLASH_META_ADDR 0x0807F000U
#define BOOT_META_MAGIC 0x314D4248U /* "HBM1" */
#define BOOT_META_VALID 0xA5A5A5A5U

typedef struct BootMeta {
    uint32_t magic;
    uint32_t valid;
    uint32_t payload_len;
    uint32_t payload_crc32;
    uint32_t version;
    uint32_t reserved[3];
} BootMeta;

static uint8_t g_rx_payload[BOOT_RX_MAX_BYTES];

static uint32_t crc32_update(uint32_t crc, uint8_t data) {
    uint32_t i;
    crc ^= (uint32_t)data;
    for (i = 0; i < 8; ++i) {
        uint32_t mask = (uint32_t)(-(int32_t)(crc & 1U));
        crc = (crc >> 1) ^ (0xEDB88320U & mask);
    }
    return crc;
}

static uint32_t crc32_calc(const uint8_t *buf, uint32_t len) {
    uint32_t i;
    uint32_t crc = 0xFFFFFFFFU;
    for (i = 0; i < len; ++i) {
        crc = crc32_update(crc, buf[i]);
    }
    return ~crc;
}

static int uart_read_byte(uint8_t *out, uint32_t timeout_ms) {
    if (HAL_UART_Receive(&UartHandle, out, 1, timeout_ms) == HAL_OK) {
        return 0;
    }
    return -1;
}

static uint32_t read_u32_le(const uint8_t *p) {
    return ((uint32_t)p[0]) |
           ((uint32_t)p[1] << 8) |
           ((uint32_t)p[2] << 16) |
           ((uint32_t)p[3] << 24);
}

static int flash_write_words(uint32_t address, const uint8_t *data, uint32_t len) {
    uint32_t i = 0;
    while (i < len) {
        uint32_t word = 0xFFFFFFFFU;
        uint32_t remain = len - i;
        uint32_t copy = remain >= 4U ? 4U : remain;
        memcpy(&word, &data[i], copy);
        if (HAL_FLASH_Program(FLASH_TYPEPROGRAM_WORD, address + i, word) != HAL_OK) {
            return -1;
        }
        i += 4U;
    }
    return 0;
}

static int flash_store_payload(const uint8_t *payload, uint32_t len, uint32_t crc) {
    FLASH_EraseInitTypeDef erase;
    uint32_t page_error = 0;
    uint32_t data_pages = (len + BOOT_FLASH_PAGE_SIZE - 1U) / BOOT_FLASH_PAGE_SIZE;
    uint8_t meta_buf[sizeof(BootMeta)];
    BootMeta meta;

    memset(&erase, 0, sizeof(erase));
    erase.TypeErase = FLASH_TYPEERASE_PAGES;
    erase.PageAddress = BOOT_FLASH_DATA_ADDR;
    erase.NbPages = data_pages;

    if (HAL_FLASH_Unlock() != HAL_OK) {
        return -1;
    }

    if (HAL_FLASHEx_Erase(&erase, &page_error) != HAL_OK) {
        HAL_FLASH_Lock();
        return -1;
    }

    if (flash_write_words(BOOT_FLASH_DATA_ADDR, payload, len) != 0) {
        HAL_FLASH_Lock();
        return -1;
    }

    erase.PageAddress = BOOT_FLASH_META_ADDR;
    erase.NbPages = 1;
    if (HAL_FLASHEx_Erase(&erase, &page_error) != HAL_OK) {
        HAL_FLASH_Lock();
        return -1;
    }

    memset(&meta, 0, sizeof(meta));
    meta.magic = BOOT_META_MAGIC;
    meta.valid = BOOT_META_VALID;
    meta.payload_len = len;
    meta.payload_crc32 = crc;
    meta.version = BOOT_PACKET_VERSION;
    memcpy(meta_buf, &meta, sizeof(meta));

    if (flash_write_words(BOOT_FLASH_META_ADDR, meta_buf, sizeof(meta_buf)) != 0) {
        HAL_FLASH_Lock();
        return -1;
    }

    HAL_FLASH_Lock();
    return 0;
}

BootUpdateResult bootloader_try_uart_update(uint32_t window_ms) {
    uint32_t start = HAL_GetTick();
    uint8_t header[10];
    uint8_t magic_window[4] = {0, 0, 0, 0};
    uint8_t b = 0;
    uint32_t payload_len;
    uint32_t packet_crc;
    uint32_t calc_crc;
    uint32_t i;

    while ((HAL_GetTick() - start) < window_ms) {
        if (uart_read_byte(&b, 10) != 0) {
            continue;
        }
        magic_window[0] = magic_window[1];
        magic_window[1] = magic_window[2];
        magic_window[2] = magic_window[3];
        magic_window[3] = b;
        if (magic_window[0] == BOOT_PACKET_MAGIC_0 &&
            magic_window[1] == BOOT_PACKET_MAGIC_1 &&
            magic_window[2] == BOOT_PACKET_MAGIC_2 &&
            magic_window[3] == BOOT_PACKET_MAGIC_3) {
            break;
        }
    }

    if ((HAL_GetTick() - start) >= window_ms) {
        return BOOT_UPDATE_TIMEOUT;
    }

    for (i = 0; i < sizeof(header); ++i) {
        if (uart_read_byte(&header[i], 1000) != 0) {
            return BOOT_UPDATE_BAD_HEADER;
        }
    }

    if (header[0] != BOOT_PACKET_VERSION) {
        return BOOT_UPDATE_BAD_HEADER;
    }

    payload_len = read_u32_le(&header[2]);
    packet_crc = read_u32_le(&header[6]);
    if (payload_len == 0U || payload_len > BOOT_RX_MAX_BYTES) {
        return BOOT_UPDATE_BAD_LENGTH;
    }

    for (i = 0; i < payload_len; ++i) {
        if (uart_read_byte(&g_rx_payload[i], 1000) != 0) {
            return BOOT_UPDATE_BAD_LENGTH;
        }
    }

    calc_crc = crc32_calc(g_rx_payload, payload_len);
    if (calc_crc != packet_crc) {
        return BOOT_UPDATE_BAD_CRC;
    }

    if (flash_store_payload(g_rx_payload, payload_len, packet_crc) != 0) {
        return BOOT_UPDATE_FLASH_ERROR;
    }
    return BOOT_UPDATE_OK;
}

int bootloader_load_program_from_flash(const uint8_t **buffer, uint32_t *size) {
    const BootMeta *meta = (const BootMeta *)BOOT_FLASH_META_ADDR;
    const uint8_t *payload = (const uint8_t *)BOOT_FLASH_DATA_ADDR;
    uint32_t crc;

    if (buffer == NULL || size == NULL) {
        return -1;
    }
    if (meta->magic != BOOT_META_MAGIC || meta->valid != BOOT_META_VALID) {
        return -1;
    }
    if (meta->payload_len == 0U || meta->payload_len > BOOT_RX_MAX_BYTES) {
        return -1;
    }

    crc = crc32_calc(payload, meta->payload_len);
    if (crc != meta->payload_crc32) {
        return -1;
    }

    *buffer = payload;
    *size = meta->payload_len;
    return 0;
}
