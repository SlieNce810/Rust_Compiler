# ESP-IDF 主组件
# 包含 ESP32-S3 平台特定代码和内置字节码数据

idf_component_register(
    SRCS "main.c"
         "vm_hal_esp32.c"
         "vm_program_data.c"
    INCLUDE_DIRS "."
    REQUIRES vm_core driver
)
