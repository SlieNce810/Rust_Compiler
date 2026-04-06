// 中间表示（IR）：把复杂的语法树变成简单的三地址码。
// 三地址码的每条指令最多有三个操作数，比如 "t1 = a + b"。
// 这样后端处理起来更简单，不用考虑复杂的嵌套表达式。

use crate::ast::{BinaryOperator, Block, Expression, Program, Statement};

// 三地址码程序 = 函数列表。
#[derive(Debug, Clone)]
pub struct ThreeAddressProgram {
    pub function_list: Vec<ThreeAddressFunction>,
}

// 三地址码函数 = 函数名 + 指令列表。
#[derive(Debug, Clone)]
pub struct ThreeAddressFunction {
    pub name: String,
    pub line_list: Vec<String>,
}

// 构建上下文：记录临时变量编号、标签编号、已生成的指令。
#[derive(Default)]
struct BuildContext {
    temp_index: usize,
    label_index: usize,
    line_list: Vec<String>,
}

// 主入口：把语法树转成三地址码。
pub fn build_three_address_code(program: &Program) -> ThreeAddressProgram {
    let mut function_list = Vec::new();

    // 每个源函数生成一段独立的三地址码，便于后端逐函数处理。
    for function in &program.function_list {
        let mut context = BuildContext::default();
        build_block_lines(&function.body_block, &mut context);

        function_list.push(ThreeAddressFunction {
            name: function.name.clone(),
            line_list: context.line_list,
        });
    }

    ThreeAddressProgram { function_list }
}

// 把代码块里的语句逐条转成三地址码。
fn build_block_lines(block: &Block, context: &mut BuildContext) {
    for statement in &block.statement_list {
        build_statement_line(statement, context);
    }
}

// 把语句转成三地址码指令。
fn build_statement_line(statement: &Statement, context: &mut BuildContext) {
    match statement {
        // 变量声明在三地址码里不需要处理，因为后端会统一扫描局部变量。
        Statement::DeclareVariable { .. } => {}

        // 赋值：先计算右边的值，再存到左边的变量。
        Statement::AssignVariable { name, value } => {
            let value_name = build_expression_value(value, context);
            context.line_list.push(format!("{name} = {value_name}"));
        }

        // return：先计算返回值，再生成 return 指令。
        Statement::ReturnValue { value } => {
            let value_name = build_expression_value(value, context);
            context.line_list.push(format!("return {value_name}"));
        }

        // if-else：翻译成条件跳转 + 无条件跳转 + 标签。
        Statement::IfElse {
            condition,
            then_block,
            else_block,
        } => {
            // if 统一翻译成：条件跳转 + then + 无条件跳转 + else + 汇合标签。
            let condition_name = build_expression_value(condition, context);
            let else_label = create_label_name(context);
            let end_label = create_label_name(context);

            // ifz = if zero，如果条件为 0 就跳转。
            context
                .line_list
                .push(format!("ifz {condition_name} goto {else_label}"));
            build_block_lines(then_block, context);
            context.line_list.push(format!("goto {end_label}"));

            context.line_list.push(format!("label {else_label}"));
            if let Some(block) = else_block {
                build_block_lines(block, context);
            }
            context.line_list.push(format!("label {end_label}"));
        }

        // while：翻译成循环头标签 + 条件跳出 + 循环体 + 回跳。
        Statement::WhileLoop {
            condition,
            body_block,
        } => {
            // while 统一翻译成：循环头标签 + 条件跳出 + 循环体 + 回跳。
            let start_label = create_label_name(context);
            let end_label = create_label_name(context);

            context.line_list.push(format!("label {start_label}"));
            let condition_name = build_expression_value(condition, context);
            context
                .line_list
                .push(format!("ifz {condition_name} goto {end_label}"));
            build_block_lines(body_block, context);
            context.line_list.push(format!("goto {start_label}"));
            context.line_list.push(format!("label {end_label}"));
        }
    }
}

// 计算表达式的值，返回值的名字（变量名、临时变量名、或立即数）。
fn build_expression_value(expression: &Expression, context: &mut BuildContext) -> String {
    match expression {
        // 变量：直接返回变量名。
        Expression::Variable(name) => name.clone(),
        // 整数：直接返回数字。
        Expression::Integer(value) => value.to_string(),
        // 浮点数：直接返回数字。
        Expression::Float(value) => value.to_string(),
        // 布尔值：转成 1 或 0。
        Expression::Bool(value) => {
            if *value {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        // 二元运算：生成临时变量来存储结果。
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let left_name = build_expression_value(left, context);
            let right_name = build_expression_value(right, context);
            let temp_name = create_temp_name(context);
            let operator_text = get_operator_text(*operator);

            // 生成一条三地址码指令，比如 "t1 = a + b"。
            context
                .line_list
                .push(format!("{temp_name} = {left_name} {operator_text} {right_name}"));

            temp_name
        }
    }
}

// 把运算符转成字符串。
fn get_operator_text(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::LessThan => "<",
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::Equal => "==",
    }
}

// 创建临时变量名：t0, t1, t2, ...
fn create_temp_name(context: &mut BuildContext) -> String {
    let name = format!("t{}", context.temp_index);
    context.temp_index += 1;
    name
}

// 创建标签名：L0, L1, L2, ...
fn create_label_name(context: &mut BuildContext) -> String {
    let name = format!("L{}", context.label_index);
    context.label_index += 1;
    name
}
