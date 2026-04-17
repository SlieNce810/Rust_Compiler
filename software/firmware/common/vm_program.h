#ifndef VM_PROGRAM_H
#define VM_PROGRAM_H

#include <stdint.h>

/* 内置字节码镜像：
 * - 由主机侧编译器产出的 .hbc 转换而来
 * - 当前用于“固件内置字节码”演示流程
 * - 下一轮动态下载可替换为运行时缓冲区
 */
extern const uint8_t g_vm_program[];
extern const uint32_t g_vm_program_size;

#endif
