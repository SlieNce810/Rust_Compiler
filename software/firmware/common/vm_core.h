#ifndef VM_CORE_H
#define VM_CORE_H

#include <stdint.h>

/* V1 解释器资源上限：与主机侧字节码约束保持一致。 */
#define VM_MAX_REG_COUNT 32
#define VM_MAX_SYMBOL_COUNT 32
#define VM_MAX_SYMBOL_NAME_LEN 31

/* 指令 mode 位定义。 */
#define VM_MODE_LHS_IMM 0x01
#define VM_MODE_RHS_IMM 0x02

/* HBC1 V1 opcode 定义（主机与端侧必须严格一致）。 */
#define VM_OP_NOP 0x00
#define VM_OP_MOV 0x01
#define VM_OP_ADD 0x02
#define VM_OP_SUB 0x03
#define VM_OP_MUL 0x04
#define VM_OP_DIV 0x05
#define VM_OP_CMP_EQ 0x06
#define VM_OP_CMP_GT 0x07
#define VM_OP_CMP_LT 0x08
#define VM_OP_JMP 0x09
#define VM_OP_JMP_IF_ZERO 0x0A
#define VM_OP_RETURN 0x0B
#define VM_OP_HALT 0x0C

/* 解释器错误码：
 * - 1~7 主要是加载/解析阶段错误
 * - 8~13 主要是执行阶段错误
 */
typedef enum VmErrorCode {
    VM_OK = 0,
    VM_ERR_BAD_ARG = 1,
    VM_ERR_BAD_MAGIC = 2,
    VM_ERR_BAD_VERSION = 3,
    VM_ERR_BAD_HEADER = 4,
    VM_ERR_SYMBOL_OVERFLOW = 5,
    VM_ERR_SYMBOL_TRUNCATED = 6,
    VM_ERR_PROGRAM_TRUNCATED = 7,
    VM_ERR_PC_OOB = 8,
    VM_ERR_REG_OOB = 9,
    VM_ERR_DIV_ZERO = 10,
    VM_ERR_BAD_OPCODE = 11,
    VM_ERR_BAD_JUMP_TARGET = 12,
    VM_ERR_STEP_LIMIT = 13
} VmErrorCode;

/* 符号表条目：将符号名映射到寄存器索引。 */
typedef struct VmSymbolEntry {
    uint8_t reg;
    char name[VM_MAX_SYMBOL_NAME_LEN + 1];
} VmSymbolEntry;

/* 指令视图：与字节码中 12 字节编码逐字段对应。 */
typedef struct VmInstruction {
    uint8_t opcode;
    uint8_t dst;
    uint8_t lhs;
    uint8_t rhs;
    uint8_t mode;
    uint8_t reserved;
    uint16_t target;
    int32_t imm;
} VmInstruction;

/* 程序视图：
 * - 前三项来自 header
 * - symbols 来自符号表区
 * - program_buffer/program_size/instruction_offset 用于定位原始字节码
 * - inst_base 指向指令区首地址
 */
typedef struct VmProgramView {
    uint8_t reg_count;
    uint16_t symbol_count;
    uint16_t instruction_count;
    VmSymbolEntry symbols[VM_MAX_SYMBOL_COUNT];
    const uint8_t *program_buffer;
    uint32_t program_size;
    uint32_t instruction_offset;
    const uint8_t *inst_base;
} VmProgramView;

/* 运行态：
 * - pc/halted/error_code/retval 描述执行状态
 * - regs 保存运算数据
 * - program 保存已加载的程序视图
 */
typedef struct VmState {
    uint16_t pc;
    int halted;
    VmErrorCode error_code;
    int32_t retval;
    int32_t regs[VM_MAX_REG_COUNT];
    VmProgramView program;
} VmState;

/* 初始化 VM 状态（不加载程序）。 */
void vm_init(VmState *vm);
/* 加载并校验字节码（header/symbol/instruction 预检）。 */
VmErrorCode vm_load_program(VmState *vm, const uint8_t *buffer, uint32_t size);
/* 执行已加载字节码，step_limit=0 表示不限制步数。 */
VmErrorCode vm_run(VmState *vm, uint32_t step_limit);

#endif
