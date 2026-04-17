# Makefile for STM32F407 firmware (端侧解释器 Demo)
#
# 使用 ARM GCC 交叉编译工具链。
# 依赖：arm-none-eabi-gcc, STM32 HAL 库（需手动放置或通过 CubeMX 生成）
#
# 用法：
#   make          -- 编译
#   make flash    -- 通过 STM32CubeProgrammer 烧录
#   make clean    -- 清理

CC = arm-none-eabi-gcc
OBJCOPY = arm-none-eabi-objcopy
SIZE = arm-none-eabi-size

MCU = -mcpu=cortex-m4 -mthumb -mfpu=fpv4-sp-d16 -mfloat-abi=hard

CFLAGS  = $(MCU) -Wall -O2 -g
CFLAGS += -DSTM32F407xx -DUSE_HAL_DRIVER
CFLAGS += -ICore/Inc
CFLAGS += -I../common

LDFLAGS = $(MCU) -TSTM32F407VGTx_FLASH.ld --specs=nano.specs --specs=nosys.specs -Wl,-Map=build/firmware.map,--cref

# 源文件
COMMON_SRCS = ../common/vm_core.c
PLATFORM_SRCS = Core/Src/main.c Core/Src/vm_hal_stm32.c Core/Src/vm_program_data.c

# 如果使用 STM32 HAL 库，需要补充以下文件（通过 CubeMX 生成）：
# HAL_SRCS = Drivers/STM32F4xx_HAL_Driver/Src/*.c
# SYSTEM_SRCS = startup_stm32f407xx.s system_stm32f4xx.c

SRCS = $(COMMON_SRCS) $(PLATFORM_SRCS)

OBJS = $(patsubst %.c, build/%.o, $(SRCS))
TARGET = build/firmware.elf
BINARY = build/firmware.bin

all: $(TARGET) $(BINARY)
	$(SIZE) $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) -o $@ $^ -lm

$(BINARY): $(TARGET)
	$(OBJCOPY) -O binary $< $@

build/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -o $@ $<

flash: $(BINARY)
	STM32_Programmer_CLI -c port=SWD -w $(BINARY) 0x08000000 -v -rst

clean:
	rm -rf build/

.PHONY: all flash clean
