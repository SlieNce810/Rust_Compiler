# MCU解释器V1设计（HBC1）

> 推荐先阅读：`Docs/03_实现设计/端侧解释器自顶向下总学习笔记.md`（先宏观后细节）

## 1. 目标

本设计用于固化端侧解释器 V1 的可执行边界，保证同一 `main.hbc` 在 `STM32F407` 与 `ESP32S3` 上行为一致。

V1 的核心原则：

- 字节码格式稳定：`magic=HBC1`，`version=1`
- 指令集合最小可执行：`imm/mov/add/sub/mul/div/cmp/goto/ifz/return`
- 端侧执行语义一致：`ifz` 统一为 `reg == 0`，`return` 写 `retval` 后停机
- 错误码可定位：每类失败映射到固定错误码

## 2. HBC1 文件格式（V1）

### 2.1 Header（固定 10 字节）

| 偏移 | 长度 | 字段 | 说明 |
| --- | --- | --- | --- |
| 0 | 4 | magic | 固定 `HBC1` |
| 4 | 1 | version | 固定 `1` |
| 5 | 1 | reg_count | 寄存器数量，`1..32` |
| 6 | 2 | symbol_count | 符号表条目数，LE |
| 8 | 2 | instruction_count | 指令条目数，LE |

### 2.2 Symbol Table（变长）

每个条目编码：

- `reg`：`u8`
- `name_len`：`u16`（LE）
- `name_bytes`：UTF-8 字节序列（不带 `\0`）

约束：

- `reg < reg_count`
- `symbol_count <= 32`
- `name_len <= 31`

### 2.3 Instruction Stream（每条 12 字节）

| 字节区间 | 字段 | 说明 |
| --- | --- | --- |
| [0] | opcode | 操作码 |
| [1] | dst | 目标寄存器 |
| [2] | lhs | 左操作数寄存器 |
| [3] | rhs | 右操作数寄存器 |
| [4] | mode | 立即数模式位 |
| [5] | reserved | 预留，V1 置 0 |
| [6..7] | target | 跳转目标 PC（LE） |
| [8..11] | imm | 立即数（`i32`，LE） |

`mode` 定义：

- `0x01`：`LHS_IMM`
- `0x02`：`RHS_IMM`

## 3. 指令集（V1）

| Opcode | 名称 | 语义 |
| --- | --- | --- |
| `0x01` | `MOV` | `dst = lhs` 或 `dst = imm` |
| `0x02` | `ADD` | `dst = left + right` |
| `0x03` | `SUB` | `dst = left - right` |
| `0x04` | `MUL` | `dst = left * right` |
| `0x05` | `DIV` | `dst = left / right`（除零报错） |
| `0x06` | `CMP_EQ` | `dst = (left == right)` |
| `0x07` | `CMP_GT` | `dst = (left > right)` |
| `0x08` | `CMP_LT` | `dst = (left < right)` |
| `0x09` | `JMP` | `pc = target` |
| `0x0A` | `JMP_IF_ZERO` | `if reg(lhs) == 0 then pc = target else pc++` |
| `0x0B` | `RETURN` | `retval = lhs/imm`，`halted = true` |
| `0x0C` | `HALT` | `halted = true` |

说明：

- V1 不使用 `NOP` 作为编译产物，但解释器保留识别能力。
- `ifz` 的唯一判定是“寄存器值是否等于 0”，不引入 truthy/falsy 扩展语义。

## 4. VM 状态模型（V1）

`VmState` 关键字段：

- `pc`：当前指令位置
- `halted`：是否停机
- `error_code`：最近错误码
- `retval`：返回值
- `regs[32]`：通用寄存器
- `program`：解析后的程序视图（header/symbol/instruction）

执行流程：

1. `vm_load_program` 校验 header 并解析 symbol table
2. 绑定 instruction view，并预检 jump target 合法性
3. `vm_run` 进入 `while (!halted)` dispatch
4. 每条指令都执行参数与边界检查
5. 完成后 `error_code=VM_OK`，失败则返回对应错误码

## 5. 错误码（V1）

| 错误码 | 名称 | 触发条件 |
| --- | --- | --- |
| 0 | `VM_OK` | 正常 |
| 1 | `VM_ERR_BAD_ARG` | 空指针或参数非法 |
| 2 | `VM_ERR_BAD_MAGIC` | 非 `HBC1` |
| 3 | `VM_ERR_BAD_VERSION` | 非 `version=1` |
| 4 | `VM_ERR_BAD_HEADER` | header 字段不合法 |
| 5 | `VM_ERR_SYMBOL_OVERFLOW` | symbol_count 超上限 |
| 6 | `VM_ERR_SYMBOL_TRUNCATED` | symbol 名字长度非法 |
| 7 | `VM_ERR_PROGRAM_TRUNCATED` | 程序数据截断 |
| 8 | `VM_ERR_PC_OOB` | `pc` 越界 |
| 9 | `VM_ERR_REG_OOB` | 寄存器索引越界 |
| 10 | `VM_ERR_DIV_ZERO` | 除零 |
| 11 | `VM_ERR_BAD_OPCODE` | 未知 opcode |
| 12 | `VM_ERR_BAD_JUMP_TARGET` | 跳转目标越界 |
| 13 | `VM_ERR_STEP_LIMIT` | 执行步数超限 |

## 6. 与主机编译器对齐约束（V1）

V1 主机产物必须满足：

- 单函数：仅允许 `main`
- 类型：仅允许 `int/bool`
- 表达式：仅允许 `imm/mov/add/sub/mul/div/cmp`
- 控制流：仅允许 `goto/ifz/return`
- 禁止浮点与函数调用

产物输出目录固定：

- `software/examples/ir/main.ir`
- `software/examples/bytecode/main.hbc`

## 7. 下一轮预留

本轮不做 bootloader 协议和动态下载实现，但保留接口：

- 端侧：`vm_load_program(vm, buffer, size)`
- 固件：预留可替换 `g_vm_program` 的程序缓冲策略

下一轮只需在传输层完成“字节码包接收 + 校验 + 调用 `vm_load_program`”，无需重写解释器核心。
