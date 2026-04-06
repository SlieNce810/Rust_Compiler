// 后端代码生成器：把三地址码翻译成目标平台的汇编代码。
// 目前支持两个平台：STM32F403（ARM 汇编）和 ESP32（Xtensa 汇编）。

use std::collections::{BTreeSet, HashMap};

use crate::ir::{ThreeAddressFunction, ThreeAddressProgram};

// 字大小：4 字节（32 位平台）。
const WORD_SIZE: i32 = 4;
// ESP32 函数帧基础大小：16 字节（保存返回地址等）。
const ESP32_BASE_FRAME: i32 = 16;

// 主入口：把三地址码程序翻译成汇编代码。
pub fn build_assembly_code(program: &ThreeAddressProgram, target_name: &str) -> String {
    let platform = get_platform(target_name);
    let mut text = String::new();

    // 在输出头写目标平台，方便肉眼确认本次后端分支是否选对。
    text.push_str(&format!("; target={}\n", platform.as_text()));
    for function in &program.function_list {
        text.push_str(&build_function_assembly(function, platform));
    }

    text
}

// 目标平台枚举。
#[derive(Clone, Copy)]
enum Platform {
    Stm32,
    Esp32,
}

impl Platform {
    // 平台名字字符串。
    fn as_text(self) -> &'static str {
        match self {
            Platform::Stm32 => "stm32f403",
            Platform::Esp32 => "esp32",
        }
    }

    // 跳转指令：STM32 用 "b"（branch），ESP32 用 "j"（jump）。
    fn jump_word(self) -> &'static str {
        match self {
            Platform::Stm32 => "b",
            Platform::Esp32 => "j",
        }
    }

    // 临时寄存器 1：用于存放第一个操作数。
    fn scratch_one(self) -> &'static str {
        match self {
            Platform::Stm32 => "r4",
            Platform::Esp32 => "a4",
        }
    }

    // 临时寄存器 2：用于存放第二个操作数。
    fn scratch_two(self) -> &'static str {
        match self {
            Platform::Stm32 => "r5",
            Platform::Esp32 => "a5",
        }
    }

    // 结果寄存器：用于存放运算结果。
    fn scratch_result(self) -> &'static str {
        match self {
            Platform::Stm32 => "r6",
            Platform::Esp32 => "a6",
        }
    }
}

// 根据目标名字选择平台，默认是 STM32。
fn get_platform(target_name: &str) -> Platform {
    if target_name.eq_ignore_ascii_case("esp32") {
        return Platform::Esp32;
    }
    Platform::Stm32
}

// 函数上下文：记录当前函数的栈布局和平台信息。
struct FunctionContext {
    platform: Platform,
    // 变量名到栈偏移的映射。
    stack_offset_by_name: HashMap<String, i32>,
    // 函数结束标签：用于 return 跳转。
    function_end_label: String,
}

// 生成单个函数的汇编代码。
fn build_function_assembly(function: &ThreeAddressFunction, platform: Platform) -> String {
    // 先扫描局部变量并分配栈偏移。这样后面每条指令都能直接查表读写栈。
    let stack_offset_by_name = get_stack_offset_by_name(function);
    let frame_size = (stack_offset_by_name.len() as i32) * WORD_SIZE;
    let function_end_label = format!("{}_epilogue", function.name);
    let context = FunctionContext {
        platform,
        stack_offset_by_name,
        function_end_label,
    };

    let mut text = String::new();
    // 函数标签。
    text.push_str(&format!("\n{}:\n", function.name));
    // prologue：保存寄存器、分配栈空间。
    text.push_str(&build_prologue(platform, frame_size));

    // 逐行翻译三地址码。
    for line in &function.line_list {
        text.push_str(&build_line_assembly(line, &context));
    }

    // epilogue：恢复寄存器、返回。
    text.push_str(&format!("{}:\n", context.function_end_label));
    text.push_str(&build_epilogue(platform, frame_size));
    text
}

// 生成单条三地址码对应的汇编代码。
// 使用字符串匹配而不是复杂的状态机，让新手更容易对照理解。
fn build_line_assembly(line: &str, context: &FunctionContext) -> String {
    // 标签：label L0
    if let Some(label_name) = line.strip_prefix("label ") {
        return format!("{}:\n", label_name);
    }
    // 无条件跳转：goto L0
    if let Some(label_name) = line.strip_prefix("goto ") {
        return build_jump_assembly(context.platform, label_name);
    }
    // 条件跳转：ifz t0 goto L0
    if let Some(condition_line) = line.strip_prefix("ifz ") {
        return build_if_zero_assembly(condition_line, context);
    }
    // 返回：return t0
    if let Some(return_value) = line.strip_prefix("return ") {
        return build_return_assembly(return_value, context);
    }
    // 赋值：t0 = a + b 或 x = 123
    if let Some((left_name, right_value)) = line.split_once(" = ") {
        return build_assign_assembly(left_name.trim(), right_value.trim(), context);
    }
    // 不支持的指令：生成注释，方便调试。
    format!("    ; unsupported: {line}\n")
}

// 生成函数开头：保存寄存器、分配栈空间。
fn build_prologue(platform: Platform, frame_size: i32) -> String {
    // ESP32 的函数开头格式不同，需要特殊处理。
    if matches!(platform, Platform::Esp32) {
        let full_size = frame_size + ESP32_BASE_FRAME;
        return format!("    entry a1, {}\n", full_size);
    }

    // STM32 的函数开头：保存寄存器 + 设置帧指针 + 分配栈空间。
    let mut text = String::new();
    text.push_str("    push {r4-r7, fp, lr}\n");
    text.push_str("    mov fp, sp\n");
    if frame_size > 0 {
        text.push_str(&format!("    sub sp, sp, #{}\n", frame_size));
    }
    text
}

// 生成函数结尾：恢复栈指针、恢复寄存器、返回。
fn build_epilogue(platform: Platform, frame_size: i32) -> String {
    // ESP32 的函数结尾格式不同。
    if matches!(platform, Platform::Esp32) {
        let full_size = frame_size + ESP32_BASE_FRAME;
        return format!("    addi a1, a1, {}\n    ret\n", full_size);
    }

    // STM32 的函数结尾：恢复栈指针 + 恢复寄存器 + 返回。
    let mut text = String::new();
    if frame_size > 0 {
        text.push_str(&format!("    add sp, sp, #{}\n", frame_size));
    }
    text.push_str("    pop {r4-r7, fp, pc}\n");
    text
}

// 生成无条件跳转指令。
fn build_jump_assembly(platform: Platform, label_name: &str) -> String {
    format!("    {} {}\n", platform.jump_word(), label_name)
}

// 生成条件跳转指令：如果条件为 0 就跳转。
fn build_if_zero_assembly(condition_line: &str, context: &FunctionContext) -> String {
    let parts = condition_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 || parts[1] != "goto" {
        return format!("    ; unsupported ifz: {condition_line}\n");
    }

    let condition_name = parts[0];
    let jump_label = parts[2];
    // 先加载条件值到寄存器。
    let condition_register = load_value(condition_name, context, context.platform.scratch_one());

    // ESP32 用 beqz（branch if equal zero）。
    if matches!(context.platform, Platform::Esp32) {
        return format!(
            "{}    beqz {}, {}\n",
            condition_register.setup_text, condition_register.value_name, jump_label
        );
    }

    // STM32 用 cmp + beq（compare + branch if equal）。
    format!(
        "{}    cmp {}, #0\n    beq {}\n",
        condition_register.setup_text, condition_register.value_name, jump_label
    )
}

// 生成返回指令：把返回值放到返回寄存器，然后跳转到函数结尾。
fn build_return_assembly(return_value: &str, context: &FunctionContext) -> String {
    let value = load_value(return_value, context, context.platform.scratch_one());

    // ESP32 用 a2 作为返回值寄存器，用 j 跳转。
    if matches!(context.platform, Platform::Esp32) {
        return format!(
            "{}    mov a2, {}\n    j {}\n",
            value.setup_text, value.value_name, context.function_end_label
        );
    }

    // STM32 用 r0 作为返回值寄存器，用 b 跳转。
    format!(
        "{}    mov r0, {}\n    b {}\n",
        value.setup_text, value.value_name, context.function_end_label
    )
}

// 生成赋值指令：可能是单值赋值，也可能是表达式赋值。
fn build_assign_assembly(left_name: &str, right_value: &str, context: &FunctionContext) -> String {
    let parts = right_value.split_whitespace().collect::<Vec<_>>();

    // 单值赋值：x = y 或 x = 123
    if parts.len() == 1 {
        let source = load_value(parts[0], context, context.platform.scratch_one());
        return save_assign_result(left_name, source, context);
    }

    if parts.len() != 3 {
        return format!("    ; unsupported assign: {right_value}\n");
    }

    // 三段式表达式：x = a + b
    // 加载左操作数。
    let left = load_value(parts[0], context, context.platform.scratch_one());
    // 加载右操作数。
    let right = load_value(parts[2], context, context.platform.scratch_two());
    let operator = parts[1];
    // 确定结果存放在哪个寄存器。
    let result_register = get_result_register(left_name, context);
    // 生成计算指令。
    let calc_text = build_calc_assembly(
        operator,
        &left.value_name,
        &right.value_name,
        &result_register,
        context.platform,
    );

    // 如果目标是临时变量，不需要保存到栈。
    if !has_stack_slot(left_name, context) {
        return format!("{}{}{}", left.setup_text, right.setup_text, calc_text);
    }

    // 如果目标是局部变量，需要保存到栈。
    let save_text = save_to_stack(left_name, &result_register, context);
    format!("{}{}{}{}", left.setup_text, right.setup_text, calc_text, save_text)
}

// 加载结果：包含加载指令和值的名字。
struct LoadedValue {
    // 加载指令（如果有）。
    setup_text: String,
    // 值的名字（寄存器名或立即数）。
    value_name: String,
}

// 加载一个值到寄存器。
fn load_value(raw_value: &str, context: &FunctionContext, scratch_register: &str) -> LoadedValue {
    // 立即数：直接内联，不需要加载到寄存器。
    if is_number(raw_value) {
        return LoadedValue {
            setup_text: String::new(),
            value_name: format!("#{}", raw_value),
        };
    }

    // 临时变量或虚拟寄存器：不在栈中，直接用虚拟寄存器名。
    if !has_stack_slot(raw_value, context) {
        return LoadedValue {
            setup_text: String::new(),
            value_name: get_virtual_register(raw_value),
        };
    }

    // 局部变量：在栈中，需要加载到寄存器。
    let offset = context.stack_offset_by_name[raw_value];
    let setup_text = if matches!(context.platform, Platform::Esp32) {
        // ESP32 用 l32i（load 32-bit immediate）。
        format!("    l32i {}, a1, {}\n", scratch_register, offset)
    } else {
        // STM32 用 ldr（load register）。
        format!("    ldr {}, [fp, #-{}]\n", scratch_register, offset)
    };

    LoadedValue {
        setup_text,
        value_name: scratch_register.to_string(),
    }
}

// 保存赋值结果。
fn save_assign_result(left_name: &str, source: LoadedValue, context: &FunctionContext) -> String {
    // 如果目标是临时变量，直接移动到虚拟寄存器。
    if !has_stack_slot(left_name, context) {
        return format!(
            "{}    mov {}, {}\n",
            source.setup_text,
            get_virtual_register(left_name),
            source.value_name
        );
    }

    // 如果目标是局部变量，保存到栈。
    let save_text = save_to_stack(left_name, &source.value_name, context);
    format!("{}{}", source.setup_text, save_text)
}

// 保存寄存器值到栈。
fn save_to_stack(name: &str, register_name: &str, context: &FunctionContext) -> String {
    let offset = context.stack_offset_by_name[name];

    if matches!(context.platform, Platform::Esp32) {
        // ESP32 用 s32i（store 32-bit immediate）。
        return format!("    s32i {}, a1, {}\n", register_name, offset);
    }

    // STM32 用 str（store register）。
    format!("    str {}, [fp, #-{}]\n", register_name, offset)
}

// 确定结果应该放在哪个寄存器。
fn get_result_register(left_name: &str, context: &FunctionContext) -> String {
    // 如果目标是局部变量，用结果寄存器。
    if has_stack_slot(left_name, context) {
        return context.platform.scratch_result().to_string();
    }

    // 如果目标是临时变量，用虚拟寄存器。
    get_virtual_register(left_name)
}

// 生成计算指令：根据平台选择不同的指令格式。
fn build_calc_assembly(
    operator: &str,
    left: &str,
    right: &str,
    result: &str,
    platform: Platform,
) -> String {
    if matches!(platform, Platform::Esp32) {
        return build_esp32_calc_assembly(operator, left, right, result);
    }

    build_stm32_calc_assembly(operator, left, right, result)
}

// 生成 ESP32 的计算指令。
fn build_esp32_calc_assembly(operator: &str, left: &str, right: &str, result: &str) -> String {
    match operator {
        "+" => format!("    add {}, {}, {}\n", result, left, right),
        "-" => format!("    sub {}, {}, {}\n", result, left, right),
        "*" => format!("    mull {}, {}, {}\n", result, left, right),
        "/" => format!("    quou {}, {}, {}\n", result, left, right),
        "<" => format!("    slt {}, {}, {}\n", result, left, right),  // set less than
        ">" => format!("    sgt {}, {}, {}\n", result, left, right),  // set greater than
        "==" => format!("    seq {}, {}, {}\n", result, left, right), // set equal
        _ => "    ; unsupported op\n".to_string(),
    }
}

// 生成 STM32 的计算指令。
fn build_stm32_calc_assembly(operator: &str, left: &str, right: &str, result: &str) -> String {
    match operator {
        "+" => format!("    add {}, {}, {}\n", result, left, right),
        "-" => format!("    sub {}, {}, {}\n", result, left, right),
        "*" => format!("    mul {}, {}, {}\n", result, left, right),
        "/" => format!("    sdiv {}, {}, {}\n", result, left, right),
        // 比较运算需要两条指令：先比较，再根据条件设置结果为 0 或 1。
        "<" => format!(
            "    cmp {}, {}\n    movlt {}, #1\n    movge {}, #0\n",
            left, right, result, result
        ),
        ">" => format!(
            "    cmp {}, {}\n    movgt {}, #1\n    movle {}, #0\n",
            left, right, result, result
        ),
        "==" => format!(
            "    cmp {}, {}\n    moveq {}, #1\n    movne {}, #0\n",
            left, right, result, result
        ),
        _ => "    ; unsupported op\n".to_string(),
    }
}

// 检查变量是否在栈中有位置。
fn has_stack_slot(name: &str, context: &FunctionContext) -> bool {
    context.stack_offset_by_name.contains_key(name)
}

// 扫描函数，为每个局部变量分配栈偏移。
fn get_stack_offset_by_name(function: &ThreeAddressFunction) -> HashMap<String, i32> {
    let local_name_list = get_local_name_list(function);
    let mut map = HashMap::new();

    // 统一按 4 字节步长分配偏移，简化后端和示例阅读。
    for (index, name) in local_name_list.iter().enumerate() {
        let offset = (index as i32 + 1) * WORD_SIZE;
        map.insert(name.clone(), offset);
    }

    map
}

// 收集函数中所有局部变量名（去重）。
fn get_local_name_list(function: &ThreeAddressFunction) -> Vec<String> {
    let mut name_set = BTreeSet::new();

    for line in &function.line_list {
        collect_names_from_line(line, &mut name_set);
    }

    name_set.into_iter().collect()
}

// 从一条三地址码中收集变量名。
fn collect_names_from_line(line: &str, name_set: &mut BTreeSet<String>) {
    // 赋值语句：左边和右边都可能是变量。
    if let Some((left_name, right_value)) = line.split_once(" = ") {
        add_name_if_local(left_name.trim(), name_set);

        for token in right_value.split_whitespace() {
            add_name_if_local(token, name_set);
        }
        return;
    }

    // return 语句：返回值可能是变量。
    if let Some(value) = line.strip_prefix("return ") {
        add_name_if_local(value.trim(), name_set);
        return;
    }

    // 条件跳转：条件可能是变量。
    if let Some(condition) = line.strip_prefix("ifz ") {
        let parts = condition.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            return;
        }
        add_name_if_local(parts[0], name_set);
    }
}

// 如果是局部变量名，就加入集合。
fn add_name_if_local(token: &str, name_set: &mut BTreeSet<String>) {
    // 数字不是变量名。
    if is_number(token) {
        return;
    }
    // 运算符不是变量名。
    if is_operator(token) {
        return;
    }
    // 临时变量不需要分配栈空间。
    if is_temp_name(token) {
        return;
    }

    name_set.insert(token.to_string());
}

// 判断是否是运算符。
fn is_operator(token: &str) -> bool {
    matches!(token, "+" | "-" | "*" | "/" | "<" | ">" | "==")
}

// 判断是否是数字。
fn is_number(token: &str) -> bool {
    token.parse::<i64>().is_ok()
}

// 判断是否是临时变量名（t0, t1, t2, ...）。
fn is_temp_name(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('t') else {
        return false;
    };
    // t 后面必须全是数字。
    !rest.is_empty() && rest.chars().all(|character| character.is_ascii_digit())
}

// 获取虚拟寄存器名。
fn get_virtual_register(name: &str) -> String {
    format!("v_{}", sanitize_name(name))
}

// 清理名字中的特殊字符，只保留字母、数字、下划线。
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                return character;
            }
            '_'
        })
        .collect()
}
