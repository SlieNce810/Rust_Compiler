#ifndef VM_HAL_H
#define VM_HAL_H

#include <stdint.h>

/* 端侧平台抽象层（HAL）最小接口：
 * VM 核心通过这些接口与硬件交互，避免把芯片细节写入 vm_core。
 */

/* 初始化板级资源（GPIO/UART/时钟等），在主入口最先调用。 */
void hal_init(void);
/* 直接设置 LED 状态（0=灭，非 0=亮）。 */
void hal_led_write(int on);
/* 翻转 LED 状态。 */
void hal_led_toggle(void);
/* 输出 key=value 整型日志，便于双平台对拍。 */
void hal_log_int(const char *key, int value);
/* 获取毫秒级时间戳，用于日志和节拍控制。 */
uint32_t hal_millis(void);
/* 毫秒阻塞延时。 */
void hal_delay_ms(uint32_t delay_ms);

#endif
