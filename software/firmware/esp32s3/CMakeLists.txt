# CMakeLists.txt for ESP32-S3 firmware (端侧解释器 Demo)
#
# 放置在 firmware/esp32s3/ 下，作为 ESP-IDF 项目的顶层 CMakeLists.txt。
# 使用前需先运行 idf.py set-target esp32s3。
#
# 目录结构要求：
#   firmware/esp32s3/
#     CMakeLists.txt       -- 本文件
#     main/
#       main.c             -- 主程序
#       vm_hal_esp32.c     -- HAL 实现
#       vm_program_data.c  -- 内置字节码
#   firmware/common/
#     vm_core.c            -- VM 核心（跨平台）
#     vm_core.h
#     vm_hal.h
#     vm_program.h

cmake_minimum_required(VERSION 3.16)

set(EXTRA_COMPONENT_DIRS
    "${CMAKE_CURRENT_SOURCE_DIR}/components"
)

# 将 common/ 下的 vm_core 作为组件引入
# 需要在 components/vm_core/ 下创建 CMakeLists.txt
include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(hopping_vm_demo)
