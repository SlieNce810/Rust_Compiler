#include "vm_core.h"
#include "vm_hal.h"

#include <stddef.h>
#include <string.h>

/* vm_core.c 负责两件事：
 * 1) 把 HBC1 字节码解析成可执行视图（vm_load_program）
 * 2) 按 V1 指令语义执行（vm_run）
 *
 * 设计原则：
 * - 先校验再运行：尽量在加载阶段拒绝坏字节码
 * - 错误码稳定：同类错误在两平台返回同一错误码
 * - 无平台依赖：本文件不直接访问 GPIO/UART 等硬件
 */

#define VM_MAGIC_0 'H'
#define VM_MAGIC_1 'B'
#define VM_MAGIC_2 'C'
#define VM_MAGIC_3 '1'
#define VM_VERSION 1
#define VM_HEADER_SIZE 10
#define VM_INSTR_SIZE 12
#define VM_NATIVE_KEY1_READ 1
#define VM_NATIVE_KEY2_READ 2
#define VM_NATIVE_SLEEP_MS 3

/* 小端读取工具：对应字节码 header/instruction 编码。 */
static uint16_t read_u16(const uint8_t *p) {
    return (uint16_t)p[0] | ((uint16_t)p[1] << 8);
}

static int32_t read_i32(const uint8_t *p) {
    return (int32_t)((uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) | ((uint32_t)p[3] << 24));
}

/* 清空执行态，不清空 program 视图。 */
static void clear_state(VmState *vm) {
    vm->pc = 0;
    vm->halted = 0;
    vm->error_code = VM_OK;
    vm->retval = 0;
    memset(vm->regs, 0, sizeof(vm->regs));
}

/* 最小 header 前置检查：指针有效且长度覆盖固定头。 */
static int is_program_header_valid(const uint8_t *buffer, uint32_t size) {
    if (buffer == NULL || size < VM_HEADER_SIZE) return 0;
    return 1;
}

/* 解析 header 并写入 program 视图。 */
static VmErrorCode parse_header(VmState *vm, const uint8_t *buffer, uint32_t size, uint32_t *cursor) {
    if (!is_program_header_valid(buffer, size)) return VM_ERR_BAD_ARG;

    if (buffer[0] != VM_MAGIC_0 || buffer[1] != VM_MAGIC_1 || buffer[2] != VM_MAGIC_2 || buffer[3] != VM_MAGIC_3) {
        return VM_ERR_BAD_MAGIC;
    }

    if (buffer[4] != VM_VERSION) return VM_ERR_BAD_VERSION;

    vm->program.reg_count = buffer[5];
    vm->program.symbol_count = read_u16(&buffer[6]);
    vm->program.instruction_count = read_u16(&buffer[8]);

    if (vm->program.reg_count == 0 || vm->program.reg_count > VM_MAX_REG_COUNT) return VM_ERR_BAD_HEADER;
    if (vm->program.symbol_count > VM_MAX_SYMBOL_COUNT) return VM_ERR_SYMBOL_OVERFLOW;
    if (vm->program.instruction_count == 0) return VM_ERR_BAD_HEADER;

    *cursor = VM_HEADER_SIZE;
    return VM_OK;
}

/* 解析符号表区，建立 reg<->name 映射。 */
static VmErrorCode parse_symbols(VmState *vm, const uint8_t *buffer, uint32_t size, uint32_t *cursor) {
    uint16_t index = 0;
    uint32_t remain;
    uint8_t reg;
    uint16_t name_len;

    while (index < vm->program.symbol_count) {
        remain = size - *cursor;
        if (remain < 3) return VM_ERR_PROGRAM_TRUNCATED;

        reg = buffer[*cursor + 0];
        name_len = read_u16(&buffer[*cursor + 1]);
        *cursor += 3;

        if (reg >= vm->program.reg_count) return VM_ERR_REG_OOB;
        if (name_len > VM_MAX_SYMBOL_NAME_LEN) return VM_ERR_SYMBOL_TRUNCATED;
        if ((size - *cursor) < name_len) return VM_ERR_PROGRAM_TRUNCATED;

        vm->program.symbols[index].reg = reg;
        memcpy(vm->program.symbols[index].name, &buffer[*cursor], name_len);
        vm->program.symbols[index].name[name_len] = '\0';
        *cursor += name_len;
        index++;
    }

    return VM_OK;
}

/* 绑定指令区视图，记录程序原始缓冲区定位信息。 */
static VmErrorCode attach_instruction_view(VmState *vm, const uint8_t *buffer, uint32_t size, uint32_t cursor) {
    uint32_t needed = (uint32_t)vm->program.instruction_count * VM_INSTR_SIZE;
    if ((size - cursor) < needed) return VM_ERR_PROGRAM_TRUNCATED;

    vm->program.program_buffer = buffer;
    vm->program.program_size = size;
    vm->program.instruction_offset = cursor;
    vm->program.inst_base = &buffer[cursor];
    return VM_OK;
}

/* 从指令区读取第 pc 条指令。 */
static VmInstruction read_instruction(const VmState *vm, uint16_t pc) {
    VmInstruction inst;
    const uint8_t *p = &vm->program.inst_base[(uint32_t)pc * VM_INSTR_SIZE];

    inst.opcode = p[0];
    inst.dst = p[1];
    inst.lhs = p[2];
    inst.rhs = p[3];
    inst.mode = p[4];
    inst.reserved = p[5];
    inst.target = read_u16(&p[6]);
    inst.imm = read_i32(&p[8]);
    return inst;
}

/* 统一寄存器边界检查。 */
static VmErrorCode check_reg(const VmState *vm, uint8_t reg) {
    if (reg >= vm->program.reg_count) return VM_ERR_REG_OOB;
    return VM_OK;
}

/* 读取寄存器值。 */
static VmErrorCode read_value(const VmState *vm, uint8_t reg, int32_t *out) {
    VmErrorCode error = check_reg(vm, reg);
    if (error != VM_OK) return error;

    *out = vm->regs[reg];
    return VM_OK;
}

/* 读取二元运算的一侧操作数：
 * - mode_flag!=0: 使用 inst.imm
 * - mode_flag==0: 使用寄存器值
 */
static VmErrorCode read_binary_side(const VmState *vm, uint8_t mode_flag, uint8_t reg, int32_t imm, int32_t *value) {
    if (mode_flag != 0) {
        *value = imm;
        return VM_OK;
    }

    return read_value(vm, reg, value);
}

/* 仅允许 LHS/RHS 两个 mode 位。 */
static int is_mode_valid(uint8_t mode) {
    uint8_t allowed = VM_MODE_LHS_IMM | VM_MODE_RHS_IMM;
    return (mode & (uint8_t)(~allowed)) == 0;
}

/* 指令格式预检：
 * 在执行前做 opcode/mode/reserved/寄存器索引合法性检查，
 * 可将多数坏包问题前置到 vm_load_program 阶段。
 */
static VmErrorCode validate_instruction_format(const VmState *vm, uint16_t pc, const VmInstruction *inst) {
    (void)pc;
    if (!is_mode_valid(inst->mode)) return VM_ERR_BAD_HEADER;

    switch (inst->opcode) {
        case VM_OP_NOP:
        case VM_OP_HALT:
            if (inst->reserved != 0) return VM_ERR_BAD_HEADER;
            return VM_OK;

        case VM_OP_MOV:
            /* MOV 支持 dst <- reg 或 dst <- imm。 */
            if (inst->reserved != 0) return VM_ERR_BAD_HEADER;
            if (check_reg(vm, inst->dst) != VM_OK) return VM_ERR_REG_OOB;
            if ((inst->mode & VM_MODE_RHS_IMM) == 0 && check_reg(vm, inst->lhs) != VM_OK) return VM_ERR_REG_OOB;
            return VM_OK;

        case VM_OP_ADD:
        case VM_OP_SUB:
        case VM_OP_MUL:
        case VM_OP_DIV:
        case VM_OP_CMP_EQ:
        case VM_OP_CMP_GT:
        case VM_OP_CMP_LT:
            /* 算术/比较类：必须有合法 dst，且每个操作数要么是寄存器要么是 imm。 */
            if (inst->reserved != 0) return VM_ERR_BAD_HEADER;
            if (check_reg(vm, inst->dst) != VM_OK) return VM_ERR_REG_OOB;
            if ((inst->mode & VM_MODE_LHS_IMM) == 0 && check_reg(vm, inst->lhs) != VM_OK) return VM_ERR_REG_OOB;
            if ((inst->mode & VM_MODE_RHS_IMM) == 0 && check_reg(vm, inst->rhs) != VM_OK) return VM_ERR_REG_OOB;
            if (inst->opcode != VM_OP_MOV && inst->mode == (VM_MODE_LHS_IMM | VM_MODE_RHS_IMM)) return VM_ERR_BAD_HEADER;
            return VM_OK;

        case VM_OP_JMP:
            if (inst->reserved != 0) return VM_ERR_BAD_HEADER;
            if (inst->mode != 0) return VM_ERR_BAD_HEADER;
            return VM_OK;

        case VM_OP_JMP_IF_ZERO:
            /* ifz 统一语义：仅允许寄存器条件（reg == 0）。 */
            if (inst->reserved != 0) return VM_ERR_BAD_HEADER;
            if (inst->mode != 0) return VM_ERR_BAD_HEADER;
            if (check_reg(vm, inst->lhs) != VM_OK) return VM_ERR_REG_OOB;
            return VM_OK;

        case VM_OP_RETURN:
            if (inst->reserved != 0) return VM_ERR_BAD_HEADER;
            if ((inst->mode & VM_MODE_RHS_IMM) != 0) return VM_ERR_BAD_HEADER;
            if ((inst->mode & VM_MODE_LHS_IMM) == 0 && check_reg(vm, inst->lhs) != VM_OK) return VM_ERR_REG_OOB;
            return VM_OK;

        case VM_OP_NATIVE:
            if (check_reg(vm, inst->dst) != VM_OK) return VM_ERR_REG_OOB;
            if (inst->target != 0) return VM_ERR_BAD_HEADER;
            if (inst->reserved == VM_NATIVE_KEY1_READ || inst->reserved == VM_NATIVE_KEY2_READ) {
                if (inst->mode != 0) return VM_ERR_BAD_HEADER;
                return VM_OK;
            }
            if (inst->reserved == VM_NATIVE_SLEEP_MS) {
                if ((inst->mode & VM_MODE_LHS_IMM) != 0) return VM_ERR_BAD_HEADER;
                if ((inst->mode & VM_MODE_RHS_IMM) == 0 && check_reg(vm, inst->lhs) != VM_OK) return VM_ERR_REG_OOB;
                return VM_OK;
            }
            return VM_ERR_BAD_HEADER;

        default:
            return VM_ERR_BAD_OPCODE;
    }
}

/* 扫描全部跳转类指令，确保 target 在 instruction_count 范围内。 */
static VmErrorCode validate_jump_targets(const VmState *vm) {
    uint16_t pc = 0;

    while (pc < vm->program.instruction_count) {
        VmInstruction inst = read_instruction(vm, pc);
        VmErrorCode format_error = validate_instruction_format(vm, pc, &inst);
        if (format_error != VM_OK) return format_error;

        if (inst.opcode == VM_OP_JMP || inst.opcode == VM_OP_JMP_IF_ZERO) {
            if (inst.target >= vm->program.instruction_count) return VM_ERR_BAD_JUMP_TARGET;
        }

        pc++;
    }

    return VM_OK;
}

/* 初始化整块状态（包括 program 视图元数据）。 */
void vm_init(VmState *vm) {
    if (vm == NULL) return;

    memset(vm, 0, sizeof(*vm));
    vm->error_code = VM_OK;
    vm->program.program_buffer = NULL;
    vm->program.program_size = 0;
    vm->program.instruction_offset = 0;
}

/* 加载流程：
 * header -> symbols -> instruction view -> format/jump 预检 -> 清空执行态
 */
VmErrorCode vm_load_program(VmState *vm, const uint8_t *buffer, uint32_t size) {
    VmErrorCode error;
    uint32_t cursor = 0;

    if (vm == NULL || buffer == NULL) return VM_ERR_BAD_ARG;

    vm_init(vm);

    error = parse_header(vm, buffer, size, &cursor);
    if (error != VM_OK) {
        vm->error_code = error;
        return error;
    }

    error = parse_symbols(vm, buffer, size, &cursor);
    if (error != VM_OK) {
        vm->error_code = error;
        return error;
    }

    error = attach_instruction_view(vm, buffer, size, cursor);
    if (error != VM_OK) {
        vm->error_code = error;
        return error;
    }

    error = validate_jump_targets(vm);
    if (error != VM_OK) {
        vm->error_code = error;
        return error;
    }

    clear_state(vm);
    return VM_OK;
}

/* 执行流程：
 * while (!halted) {
 *   fetch(inst) -> dispatch(opcode) -> 更新 pc/retval/error_code
 * }
 */
VmErrorCode vm_run(VmState *vm, uint32_t step_limit) {
    uint32_t step = 0;

    if (vm == NULL) return VM_ERR_BAD_ARG;
    if (vm->program.inst_base == NULL) return VM_ERR_BAD_ARG;

    while (!vm->halted) {
        VmInstruction inst;
        VmErrorCode error;
        int32_t left = 0;
        int32_t right = 0;
        int32_t value = 0;

        if (vm->pc >= vm->program.instruction_count) {
            vm->error_code = VM_ERR_PC_OOB;
            return vm->error_code;
        }

        inst = read_instruction(vm, vm->pc);

        switch (inst.opcode) {
            case VM_OP_NOP:
                /* 空操作，仅推进 pc。 */
                vm->pc++;
                break;

            case VM_OP_MOV:
                /* MOV: dst <- imm / dst <- reg */
                error = check_reg(vm, inst.dst);
                if (error != VM_OK) {
                    vm->error_code = error;
                    return error;
                }

                if ((inst.mode & VM_MODE_RHS_IMM) != 0) {
                    vm->regs[inst.dst] = inst.imm;
                } else {
                    error = read_value(vm, inst.lhs, &value);
                    if (error != VM_OK) {
                        vm->error_code = error;
                        return error;
                    }
                    vm->regs[inst.dst] = value;
                }

                vm->pc++;
                break;

            case VM_OP_ADD:
            case VM_OP_SUB:
            case VM_OP_MUL:
            case VM_OP_DIV:
            case VM_OP_CMP_EQ:
            case VM_OP_CMP_GT:
            case VM_OP_CMP_LT:
                /* 二元运算/比较：先取两侧，再写 dst。 */
                error = check_reg(vm, inst.dst);
                if (error != VM_OK) {
                    vm->error_code = error;
                    return error;
                }

                error = read_binary_side(vm, inst.mode & VM_MODE_LHS_IMM, inst.lhs, inst.imm, &left);
                if (error != VM_OK) {
                    vm->error_code = error;
                    return error;
                }

                error = read_binary_side(vm, inst.mode & VM_MODE_RHS_IMM, inst.rhs, inst.imm, &right);
                if (error != VM_OK) {
                    vm->error_code = error;
                    return error;
                }

                if (inst.opcode == VM_OP_ADD) vm->regs[inst.dst] = left + right;
                if (inst.opcode == VM_OP_SUB) vm->regs[inst.dst] = left - right;
                if (inst.opcode == VM_OP_MUL) vm->regs[inst.dst] = left * right;
                if (inst.opcode == VM_OP_DIV) {
                    if (right == 0) {
                        vm->error_code = VM_ERR_DIV_ZERO;
                        return vm->error_code;
                    }
                    vm->regs[inst.dst] = left / right;
                }
                if (inst.opcode == VM_OP_CMP_EQ) vm->regs[inst.dst] = (left == right) ? 1 : 0;
                if (inst.opcode == VM_OP_CMP_GT) vm->regs[inst.dst] = (left > right) ? 1 : 0;
                if (inst.opcode == VM_OP_CMP_LT) vm->regs[inst.dst] = (left < right) ? 1 : 0;

                vm->pc++;
                break;

            case VM_OP_JMP:
                /* 无条件跳转。 */
                vm->pc = inst.target;
                break;

            case VM_OP_JMP_IF_ZERO:
                /* 条件跳转：value==0 则跳转，否则顺序执行。 */
                error = read_value(vm, inst.lhs, &value);
                if (error != VM_OK) {
                    vm->error_code = error;
                    return error;
                }

                if (value == 0) vm->pc = inst.target;
                else vm->pc++;
                break;

            case VM_OP_RETURN:
                /* RETURN: 写 retval 并停机。 */
                if ((inst.mode & VM_MODE_LHS_IMM) != 0) {
                    vm->retval = inst.imm;
                } else {
                    error = read_value(vm, inst.lhs, &value);
                    if (error != VM_OK) {
                        vm->error_code = error;
                        return error;
                    }
                    vm->retval = value;
                }

                vm->halted = 1;
                vm->pc++;
                break;

            case VM_OP_HALT:
                /* HALT: 仅停机。 */
                vm->halted = 1;
                vm->pc++;
                break;

            case VM_OP_NATIVE:
                error = check_reg(vm, inst.dst);
                if (error != VM_OK) {
                    vm->error_code = error;
                    return error;
                }

                if (inst.reserved == VM_NATIVE_KEY1_READ) {
                    vm->regs[inst.dst] = hal_key1_read() ? 1 : 0;
                    vm->pc++;
                    break;
                }
                if (inst.reserved == VM_NATIVE_KEY2_READ) {
                    vm->regs[inst.dst] = hal_key2_read() ? 1 : 0;
                    vm->pc++;
                    break;
                }
                if (inst.reserved == VM_NATIVE_SLEEP_MS) {
                    if ((inst.mode & VM_MODE_RHS_IMM) != 0) {
                        value = inst.imm;
                    } else {
                        error = read_value(vm, inst.lhs, &value);
                        if (error != VM_OK) {
                            vm->error_code = error;
                            return error;
                        }
                    }
                    if (value < 0) value = 0;
                    hal_delay_ms((uint32_t)value);
                    vm->regs[inst.dst] = 0;
                    vm->pc++;
                    break;
                }

                vm->error_code = VM_ERR_BAD_OPCODE;
                return vm->error_code;

            default:
                vm->error_code = VM_ERR_BAD_OPCODE;
                return vm->error_code;
        }

        step++;
        if (step_limit > 0 && step > step_limit) {
            vm->error_code = VM_ERR_STEP_LIMIT;
            return vm->error_code;
        }
    }

    vm->error_code = VM_OK;
    return VM_OK;
}
