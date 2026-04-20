#include "bootloader_hbc_esp32.h"

#include "driver/uart.h"
#include "esp_partition.h"
#include "esp_timer.h"
#include "esp_system.h"
#include "freertos/FreeRTOS.h"

#include <string.h>

#define ESP_BOOT_PACKET_MAGIC_0 'H'
#define ESP_BOOT_PACKET_MAGIC_1 'U'
#define ESP_BOOT_PACKET_MAGIC_2 'P'
#define ESP_BOOT_PACKET_MAGIC_3 '1'
#define ESP_BOOT_PACKET_VERSION 1U

#define ESP_BOOT_UART_NUM UART_NUM_0
#define ESP_BOOT_MAX_BYTES 16384U
#define ESP_BOOT_SLOT_LABEL "hbc_slot"
#define ESP_BOOT_META_MAGIC 0x324D4248U /* "HBM2" */
#define ESP_BOOT_META_VALID 0xA5A5A5A5U
#define ESP_BOOT_META_OFFSET 0U
#define ESP_BOOT_PAYLOAD_OFFSET 4096U

typedef struct EspBootMeta {
    uint32_t magic;
    uint32_t valid;
    uint32_t payload_len;
    uint32_t payload_crc32;
    uint32_t version;
    uint32_t reserved[3];
} EspBootMeta;

static uint8_t g_rx_payload[ESP_BOOT_MAX_BYTES];
static uint8_t g_loaded_payload[ESP_BOOT_MAX_BYTES];

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
    int rd = uart_read_bytes(ESP_BOOT_UART_NUM, out, 1, pdMS_TO_TICKS(timeout_ms));
    return rd == 1 ? 0 : -1;
}

static uint32_t read_u32_le(const uint8_t *p) {
    return ((uint32_t)p[0]) |
           ((uint32_t)p[1] << 8) |
           ((uint32_t)p[2] << 16) |
           ((uint32_t)p[3] << 24);
}

static const esp_partition_t *find_slot_partition(void) {
    return esp_partition_find_first(ESP_PARTITION_TYPE_DATA, ESP_PARTITION_SUBTYPE_ANY, ESP_BOOT_SLOT_LABEL);
}

EspBootUpdateResult esp_bootloader_try_uart_update(uint32_t window_ms) {
    uint32_t start = (uint32_t)(esp_timer_get_time() / 1000ULL);
    const esp_partition_t *part = find_slot_partition();
    uint8_t header[10];
    uint8_t magic_window[4] = {0, 0, 0, 0};
    uint8_t b = 0;
    uint32_t payload_len;
    uint32_t packet_crc;
    uint32_t calc_crc;
    uint32_t i;
    EspBootMeta meta;

    if (part == NULL) {
        return ESP_BOOT_UPDATE_NO_PARTITION;
    }

    while (((uint32_t)(esp_timer_get_time() / 1000ULL) - start) < window_ms) {
        if (uart_read_byte(&b, 10) != 0) {
            continue;
        }
        magic_window[0] = magic_window[1];
        magic_window[1] = magic_window[2];
        magic_window[2] = magic_window[3];
        magic_window[3] = b;
        if (magic_window[0] == ESP_BOOT_PACKET_MAGIC_0 &&
            magic_window[1] == ESP_BOOT_PACKET_MAGIC_1 &&
            magic_window[2] == ESP_BOOT_PACKET_MAGIC_2 &&
            magic_window[3] == ESP_BOOT_PACKET_MAGIC_3) {
            break;
        }
    }

    if (((uint32_t)(esp_timer_get_time() / 1000ULL) - start) >= window_ms) {
        return ESP_BOOT_UPDATE_TIMEOUT;
    }

    for (i = 0; i < sizeof(header); ++i) {
        if (uart_read_byte(&header[i], 1000) != 0) {
            return ESP_BOOT_UPDATE_BAD_HEADER;
        }
    }

    if (header[0] != ESP_BOOT_PACKET_VERSION) {
        return ESP_BOOT_UPDATE_BAD_HEADER;
    }

    payload_len = read_u32_le(&header[2]);
    packet_crc = read_u32_le(&header[6]);
    if (payload_len == 0U || payload_len > ESP_BOOT_MAX_BYTES) {
        return ESP_BOOT_UPDATE_BAD_LENGTH;
    }
    if ((ESP_BOOT_PAYLOAD_OFFSET + payload_len) > part->size) {
        return ESP_BOOT_UPDATE_BAD_LENGTH;
    }

    for (i = 0; i < payload_len; ++i) {
        if (uart_read_byte(&g_rx_payload[i], 1000) != 0) {
            return ESP_BOOT_UPDATE_BAD_LENGTH;
        }
    }

    calc_crc = crc32_calc(g_rx_payload, payload_len);
    if (calc_crc != packet_crc) {
        return ESP_BOOT_UPDATE_BAD_CRC;
    }

    if (esp_partition_erase_range(part, 0, part->size) != ESP_OK) {
        return ESP_BOOT_UPDATE_FLASH_ERROR;
    }
    if (esp_partition_write(part, ESP_BOOT_PAYLOAD_OFFSET, g_rx_payload, payload_len) != ESP_OK) {
        return ESP_BOOT_UPDATE_FLASH_ERROR;
    }

    memset(&meta, 0, sizeof(meta));
    meta.magic = ESP_BOOT_META_MAGIC;
    meta.valid = ESP_BOOT_META_VALID;
    meta.payload_len = payload_len;
    meta.payload_crc32 = packet_crc;
    meta.version = ESP_BOOT_PACKET_VERSION;
    if (esp_partition_write(part, ESP_BOOT_META_OFFSET, &meta, sizeof(meta)) != ESP_OK) {
        return ESP_BOOT_UPDATE_FLASH_ERROR;
    }
    return ESP_BOOT_UPDATE_OK;
}

int esp_bootloader_load_program_from_flash(const uint8_t **buffer, uint32_t *size) {
    const esp_partition_t *part = find_slot_partition();
    EspBootMeta meta;
    uint32_t crc;

    if (buffer == NULL || size == NULL || part == NULL) {
        return -1;
    }
    if (esp_partition_read(part, ESP_BOOT_META_OFFSET, &meta, sizeof(meta)) != ESP_OK) {
        return -1;
    }
    if (meta.magic != ESP_BOOT_META_MAGIC || meta.valid != ESP_BOOT_META_VALID) {
        return -1;
    }
    if (meta.payload_len == 0U || meta.payload_len > ESP_BOOT_MAX_BYTES) {
        return -1;
    }
    if ((ESP_BOOT_PAYLOAD_OFFSET + meta.payload_len) > part->size) {
        return -1;
    }
    if (esp_partition_read(part, ESP_BOOT_PAYLOAD_OFFSET, g_loaded_payload, meta.payload_len) != ESP_OK) {
        return -1;
    }

    crc = crc32_calc(g_loaded_payload, meta.payload_len);
    if (crc != meta.payload_crc32) {
        return -1;
    }
    *buffer = g_loaded_payload;
    *size = meta.payload_len;
    return 0;
}
