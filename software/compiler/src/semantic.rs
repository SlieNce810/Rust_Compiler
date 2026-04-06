// 语义分析器：检查类型错误、变量未声明等问题。
// 这一步在编译阶段尽早暴露源代码错误，避免生成错误的汇编代码。

use std::collections::HashMap;

use crate::ast::{BinaryOperator, Block, Expression, Function, Program, Statement, TypeName};
use crate::error::CompileResult;

// 主入口：检查整个程序。
pub fn check_program(program: &Program) -> CompileResult<()> {
    // 每个函数独立检查，避免一个函数的符号污染另一个函数。
    for function in &program.function_list {
        check_function(function)?;
    }
    Ok(())
}

// 检查单个函数。
fn check_function(function: &Function) -> CompileResult<()> {
    // 符号表：记录每个变量名对应的类型。
    let mut type_by_name = HashMap::<String, TypeName>::new();

    // 先把参数加入符号表，因为函数体内可以用参数。
    for parameter in &function.parameter_list {
        type_by_name.insert(parameter.name.clone(), parameter.type_name);
    }

    check_block(&function.body_block, &type_by_name, function.return_type)
}

// 检查代码块。
// 为什么需要 parent_type_by_name？因为内层代码块可以访问外层变量。
fn check_block(block: &Block, parent_type_by_name: &HashMap<String, TypeName>, return_type: TypeName) -> CompileResult<()> {
    // 进入子块时复制一份符号表，模拟"块级作用域"。
    // 这样内层声明的变量不会影响外层。
    let mut type_by_name = parent_type_by_name.clone();

    for statement in &block.statement_list {
        check_statement(statement, &mut type_by_name, return_type)?;
    }

    Ok(())
}

// 检查语句。
fn check_statement(statement: &Statement, type_by_name: &mut HashMap<String, TypeName>, return_type: TypeName) -> CompileResult<()> {
    match statement {
        // 声明变量：检查是否重复声明。
        Statement::DeclareVariable { type_name, name } => {
            if type_by_name.contains_key(name) {
                return Err(format!("redeclaration of variable: {name}"));
            }
            type_by_name.insert(name.clone(), *type_name);
            Ok(())
        }
        // 赋值：检查变量是否声明、类型是否匹配。
        Statement::AssignVariable { name, value } => {
            let Some(variable_type) = type_by_name.get(name).copied() else {
                return Err(format!("use before declaration: {name}"));
            };

            let value_type = get_expression_type(value, type_by_name)?;
            if variable_type != value_type {
                return Err(format!("type mismatch in assignment to {name}: expected {variable_type}, got {value_type}"));
            }
            Ok(())
        }
        // if 语句：检查条件是否为 bool 类型。
        Statement::IfElse {
            condition,
            then_block,
            else_block,
        } => {
            // if 条件必须是 bool，防止把整数当条件误用。
            let condition_type = get_expression_type(condition, type_by_name)?;
            if condition_type != TypeName::Bool {
                return Err("if condition must be bool".to_string());
            }

            check_block(then_block, type_by_name, return_type)?;
            if let Some(block) = else_block {
                check_block(block, type_by_name, return_type)?;
            }
            Ok(())
        }
        // while 循环：检查条件是否为 bool 类型。
        Statement::WhileLoop {
            condition,
            body_block,
        } => {
            let condition_type = get_expression_type(condition, type_by_name)?;
            if condition_type != TypeName::Bool {
                return Err("while condition must be bool".to_string());
            }

            check_block(body_block, type_by_name, return_type)
        }
        // return 语句：检查返回值类型是否匹配。
        Statement::ReturnValue { value } => {
            let value_type = get_expression_type(value, type_by_name)?;
            if value_type != return_type {
                return Err(format!("return type mismatch: expected {return_type}, got {value_type}"));
            }
            Ok(())
        }
    }
}

// 获取表达式的类型。
fn get_expression_type(expression: &Expression, type_by_name: &HashMap<String, TypeName>) -> CompileResult<TypeName> {
    match expression {
        // 变量：从符号表查类型。
        Expression::Variable(name) => type_by_name
            .get(name)
            .copied()
            .ok_or_else(|| format!("undeclared variable: {name}")),

        // 字面量：类型由字面量本身决定。
        Expression::Integer(_) => Ok(TypeName::Int),
        Expression::Float(_) => Ok(TypeName::Float),
        Expression::Bool(_) => Ok(TypeName::Bool),

        // 二元运算：需要检查左右类型是否兼容。
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let left_type = get_expression_type(left, type_by_name)?;
            let right_type = get_expression_type(right, type_by_name)?;
            check_binary_type(left_type, *operator, right_type)
        }
    }
}

// 检查二元运算的类型规则。
fn check_binary_type(left_type: TypeName, operator: BinaryOperator, right_type: TypeName) -> CompileResult<TypeName> {
    // 这里集中处理二元运算规则，避免散落在多个分支里难维护。
    match operator {
        // 加减乘除：两边必须是相同类型的数字。
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide => check_number_math_type(left_type, right_type),

        // 大于小于：两边必须是相同类型的数字，结果是 bool。
        BinaryOperator::LessThan | BinaryOperator::GreaterThan => {
            check_number_compare_type(left_type, right_type)
        }

        // 相等判断：两边类型必须相同，结果是 bool。
        BinaryOperator::Equal => {
            if left_type != right_type {
                return Err(format!("equality type mismatch: {left_type} vs {right_type}"));
            }
            Ok(TypeName::Bool)
        }
    }
}

// 检查算术运算的类型：两边必须是相同类型的 int 或 float。
fn check_number_math_type(left_type: TypeName, right_type: TypeName) -> CompileResult<TypeName> {
    if left_type != right_type {
        return Err(format!("arithmetic type mismatch: {left_type} vs {right_type}"));
    }

    if left_type != TypeName::Int && left_type != TypeName::Float {
        return Err(format!("arithmetic needs int or float, got {left_type}"));
    }

    Ok(left_type)
}

// 检查比较运算的类型：两边必须是相同类型的 int 或 float，结果是 bool。
fn check_number_compare_type(left_type: TypeName, right_type: TypeName) -> CompileResult<TypeName> {
    if left_type != right_type {
        return Err(format!("comparison type mismatch: {left_type} vs {right_type}"));
    }

    if left_type != TypeName::Int && left_type != TypeName::Float {
        return Err(format!("comparison needs int or float, got {left_type}"));
    }

    Ok(TypeName::Bool)
}
