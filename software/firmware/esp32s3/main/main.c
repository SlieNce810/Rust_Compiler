/* main.c -- ESP32-S3 端侧解释器 Demo 主程序
 *
 * 流程：
 * 1. HAL 初始化（GPIO / UART）
 * 2. 加载内置 blink.hbc 字节码
 * 3. 循环执行 VM：每次执行完毕后根据 retval 切换 LED
 * 4. 串口输出启动日志、状态切换、异常码
 */

#include "vm_core.h"
#include "vm_hal.h"
#include "vm_program.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

void app_main(void) {
    VmState vm;
    VmErrorCode error;
    int led_on = 0;

    /* 1. HAL 初始化 */
    hal_init();

    hal_log_int("vm_boot", 1);

    /* 2. 加载内置字节码 */
    error = vm_load_program(&vm, g_vm_program, g_vm_program_size);
    if (error != VM_OK) {
        hal_log_int("vm_load_err", (int)error);
        return;
    }

    hal_log_int("vm_loaded", 1);

    /* 3. 主循环：反复执行 VM，根据 retval 控制 LED */
    while (1) {
        /* 重新加载程序（重置 PC 和寄存器） */
        error = vm_load_program(&vm, g_vm_program, g_vm_program_size);
        if (error != VM_OK) {
            hal_log_int("vm_load_err", (int)error);
            hal_delay_ms(2000);
            continue;
        }

        /* 执行，步数限制防止死循环 */
        error = vm_run(&vm, 100000);

        if (error != VM_OK) {
            hal_log_int("vm_run_err", (int)error);
            hal_delay_ms(2000);
            continue;
        }

        hal_log_int("vm_retval", vm.retval);

        /* 根据 retval 切换 LED */
        if (vm.retval != 0) {
            led_on = !led_on;
            hal_led_write(led_on);
            hal_log_int("led_on", led_on);
            hal_log_int("tick_ms", (int)hal_millis());
        }

        /* 每轮之间短暂延时 */
        hal_delay_ms(100);
    }
}
