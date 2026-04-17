# examples 目录说明

本目录按“输入/输出/历史文件”分组，避免源文件和编译产物混放。

- `source/`：`.hopping` 源代码输入
- `asm/`：汇编输出（`.asm`）
- `ir/`：IR 文本输出（`.ir`）
- `bytecode/`：字节码输出（`.hbc`）
- `notes/`：示例说明文档
- `legacy/`：历史 `.mc` 文件（兼容保留，不建议继续新增）

推荐使用方式：

1. 编辑 `source/*.hopping`
2. 编译输出到 `asm/`、`ir/`、`bytecode/`
3. 只在需要回溯兼容时查看 `legacy/`
