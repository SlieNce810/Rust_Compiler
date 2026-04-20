/* main.c -- STM32F103 指南者端侧解释器 Demo
 *
 * 运行流程：
 * 1. 初始化时钟节拍（SysTick 1ms）与板级 HAL
 * 2. 加载内置 main.hbc 字节码
 * 3. 循环执行 VM，根据 retval 翻转 LED，并把日志显示到串口+LCD
 */

#include "vm_core.h"
#include "vm_hal.h"
#include "vm_program.h"

#include "stm32f10x.h"

int main(void) {
    VmState vm;
    VmErrorCode error;
    int led_on = 0;

    SystemInit();
    SysTick_Config(SystemCoreClock / 1000U);
    hal_init();

    hal_log_int("vm_boot", 1);

    error = vm_load_program(&vm, g_vm_program, g_vm_program_size);
    if (error != VM_OK) {
        hal_log_int("vm_load_err", (int)error);
        while (1) {
        }
    }

    hal_log_int("vm_loaded", 1);

    while (1) {
        error = vm_load_program(&vm, g_vm_program, g_vm_program_size);
        if (error != VM_OK) {
            hal_log_int("vm_load_err", (int)error);
            hal_delay_ms(1000);
            continue;
        }

        error = vm_run(&vm, 100000);
        if (error != VM_OK) {
            hal_log_int("vm_run_err", (int)error);
            hal_delay_ms(1000);
            continue;
        }

        hal_log_int("vm_retval", vm.retval);

        if (vm.retval != 0) {
            led_on = !led_on;
            hal_led_write(led_on);
            hal_log_int("led_on", led_on);
            hal_log_int("tick_ms", (int)hal_millis());
        }

        hal_delay_ms(100);
    }
}
