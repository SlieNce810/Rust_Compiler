# AI只读前端展示页规划（可选）

## 1. 目标

- 仅做编译结果与 AI 解释的可视化展示，不参与编译裁决。
- 强调“学习与分析”，不提供“自然语言直接生成最终 Hopping 程序”的提交按钮。

## 2. 数据来源

- 源码：`examples/source/*.hopping`
- IR：`examples/ir/*.ir`
- 字节码：`examples/bytecode/*.hbc`（前端只做摘要展示）
- AI 输出：
  - 失败场景：编译器终端即时建议（不落失败 md）
  - 成功场景：可选 AI 报告 `examples/notes/ai_report_*.md`

## 3. 页面结构（三栏只读）

1. 左栏：源码视图（Hopping）
2. 中栏：IR 与字节码摘要
3. 右栏：AI 解释报告（Markdown 渲染）

页面原则：
- 所有内容来自文件读取结果，不开放在线编辑回写。
- 不提供“生成并覆盖源码”的快捷入口。
- 若失败场景需要展示，建议前端接入“终端日志摘要”而不是依赖失败 md 文件。

## 4. 后端接口建议（本阶段仅规划）

- `GET /api/source?file=main.hopping`
- `GET /api/ir?file=main.ir`
- `GET /api/bytecode/summary?file=main.hbc`
- `GET /api/ai-report?file=ai_report_success.md`
- `GET /api/terminal-ai-guidance?task=compile-fail`（可选，读取终端建议摘要）

说明：
- 接口只读，禁止写接口。
- 需要路径白名单，避免任意文件读取风险。

## 5. 安全与边界

- 前端展示层不是语言语义层，不得修改编译器行为。
- AI 结果必须显示免责声明：建议仅供参考，以编译器输出为准。
- 若后续加入在线执行，仅允许读取沙箱示例数据，不直接驱动 MCU 侧执行链路。

## 6. 验收标准（展示页）

- 能读取并展示成功场景 AI 报告。
- 能展示失败场景终端建议摘要（或明确仅在终端查看）。
- 能关联展示对应源码与 IR（成功场景）。
- 默认无“自动生成最终程序”的功能入口。
