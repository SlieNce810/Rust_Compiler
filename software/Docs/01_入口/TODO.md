# TODO

## 文档入口（自顶向下）

建议按以下顺序阅读并执行：

1. `Docs/03_实现设计/端侧解释器自顶向下总学习笔记.md`
2. `Docs/03_实现设计/MCU解释器V1设计.md`
3. `Docs/03_实现设计/端侧Demo运行手册.md`
4. `Docs/01_入口/TODO.md`（本文件）

## 本轮可交付（端侧解释器 Demo V1）

### 已完成

- [x] 固化 `HBC1` 字节码 `v1`（`magic/version/opcode/error_code`）
- [x] 主机侧补齐 V1 子集约束（单函数 `main`、`int/bool`、无浮点）
- [x] 产物路径固定：`examples/ir/main.ir`、`examples/bytecode/main.hbc`
- [x] 双平台共用 VM 核心并统一执行语义（`ifz`、`return`、错误码）
- [x] STM32F407 + ESP32S3 双平台点灯与串口日志统一口径
- [x] 新增 V1 设计文档与端侧 Demo 运行手册

### 本轮验收标准

- [ ] 主机侧：`cargo check` 通过，能生成 `main.ir/main.hbc`
- [ ] V1 约束生效：非法输入能被编译器拒绝并给出可读报错
- [ ] STM32F407：上电后有 `vm_boot/vm_loaded`，LED 可见翻转，异常打印 `vm_run_err`
- [ ] ESP32S3：同样日志与 LED 行为，输出序列与 STM32 一致（忽略时间戳）

## 下轮（bootloader 协议与动态下载）

- [ ] 完成 `Bootloader输入清单.md` 的板级信息补全
- [ ] 定义下载包协议（版本、长度、CRC、错误码、兼容策略）
- [ ] 在端侧接入串口动态下载并调用 `vm_load_program()`
- [ ] 增加校验失败回滚与缓冲区越界保护
- [ ] 预留 BLE/WiFi 传输通道（复用下载包协议）

## 节奏原则

1. 本轮先保证“同一字节码双平台一致执行”
2. 下轮再接入动态下载，避免并行复杂度失控
