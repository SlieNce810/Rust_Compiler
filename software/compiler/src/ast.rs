// 这里的数据结构是编译器的"骨架"：词法分析器产出的 token 会变成这些结构。
// 所有结构都用最简单的方式定义，没有继承、没有泛型，让新手能直接看懂字段含义。

use std::fmt;

// 类型名字：只支持三种基础类型，简化编译器实现难度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeName {
    Int,
    Float,
    Bool,
}

// 让类型名字能直接打印成字符串，方便报错时显示。
impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeName::Int => write!(formatter, "int"),
            TypeName::Float => write!(formatter, "float"),
            TypeName::Bool => write!(formatter, "bool"),
        }
    }
}

// 程序 = 函数列表。不允许可执行语句在函数外，简化编译流程。
#[derive(Debug, Clone)]
pub struct Program {
    pub function_list: Vec<Function>,
}

// 函数 = 名字 + 返回类型 + 参数列表 + 函数体。
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: TypeName,
    pub parameter_list: Vec<Parameter>,
    pub body_block: Block,
}

// 参数 = 类型 + 名字，顺序和源代码一致。
#[derive(Debug, Clone)]
pub struct Parameter {
    pub type_name: TypeName,
    pub name: String,
}

// 代码块 = 语句列表。用大括号包围的部分就是一个块。
#[derive(Debug, Clone)]
pub struct Block {
    pub statement_list: Vec<Statement>,
}

// 语句 = 五种之一。每种语句对应源代码里的一行或一个结构。
#[derive(Debug, Clone)]
pub enum Statement {
    // 声明变量：int x;
    DeclareVariable { type_name: TypeName, name: String },
    // 赋值：x = 123;
    AssignVariable { name: String, value: Expression },
    // if-else 结构：else 部分可以没有，所以用 Option。
    IfElse {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    // while 循环。
    WhileLoop { condition: Expression, body_block: Block },
    // return 语句。
    ReturnValue { value: Expression },
}

// 表达式 = 可以计算出一个值的东西。
#[derive(Debug, Clone)]
pub enum Expression {
    // 变量名：x
    Variable(String),
    // 整数字面量：123
    Integer(i64),
    // 浮点字面量：3.14
    Float(f64),
    // 布尔字面量：true / false
    Bool(bool),
    // 二元运算：a + b。用 Box 包装是为了避免无限大小的递归类型。
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
}

// 二元运算符：加减乘除、比较、相等判断。
#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    LessThan,
    GreaterThan,
    Equal,
}
