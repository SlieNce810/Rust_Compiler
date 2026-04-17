# ESP-IDF 组件: vm_core
# 将 firmware/common/ 下的平台无关 VM 代码封装为 ESP-IDF 组件

idf_component_register(
    SRCS "${CMAKE_CURRENT_SOURCE_DIR}/../../common/vm_core.c"
    INCLUDE_DIRS "${CMAKE_CURRENT_SOURCE_DIR}/../../common"
)
