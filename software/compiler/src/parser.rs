// 语法分析器：把 token 列表变成语法树（AST）。
// 比如把 [Int, Name("x"), Assign, Integer(123), Semicolon] 变成 DeclareVariable 节点。

use crate::ast::{
    BinaryOperator, Block, Expression, Function, Parameter, Program, Statement, TypeName,
};
use crate::error::CompileResult;
use crate::lexer::Token;

// Parser 维护当前读到哪个 token 了，用下标记录位置。
pub struct Parser {
    token_list: Vec<Token>,
    current_index: usize,
}

impl Parser {
    pub fn new(token_list: Vec<Token>) -> Self {
        Self {
            token_list,
            current_index: 0,
        }
    }

    // 主入口：解析整个程序。
    pub fn parse_program(&mut self) -> CompileResult<Program> {
        let mut function_list = Vec::new();

        // 顶层只允许函数定义，读到文件结束为止。
        while !self.is_current(Token::EndOfFile) {
            function_list.push(self.parse_function()?);
        }

        Ok(Program { function_list })
    }

    // 解析函数定义：func int add(int a, int b) { ... }
    fn parse_function(&mut self) -> CompileResult<Function> {
        self.expect(Token::Func)?;

        let return_type = self.parse_type_name()?;
        let function_name = self.parse_name()?;
        let parameter_list = self.parse_parameter_list()?;
        let body_block = self.parse_block()?;

        Ok(Function {
            name: function_name,
            return_type,
            parameter_list,
            body_block,
        })
    }

    // 解析参数列表：(int a, int b) 或 ()
    fn parse_parameter_list(&mut self) -> CompileResult<Vec<Parameter>> {
        self.expect(Token::LeftParen)?;

        let mut parameter_list = Vec::new();
        // 空参数函数：func int main() {}，直接返回空列表。
        if self.is_current(Token::RightParen) {
            self.advance();
            return Ok(parameter_list);
        }

        // 读第一个参数。
        loop {
            let type_name = self.parse_type_name()?;
            let name = self.parse_name()?;
            parameter_list.push(Parameter { type_name, name });

            // 有逗号说明还有参数，继续读；没有逗号说明结束了。
            if !self.is_current(Token::Comma) {
                break;
            }
            self.advance();
        }

        self.expect(Token::RightParen)?;
        Ok(parameter_list)
    }

    // 解析代码块：{ 语句1; 语句2; ... }
    fn parse_block(&mut self) -> CompileResult<Block> {
        self.expect(Token::LeftBrace)?;

        let mut statement_list = Vec::new();
        // 一直读语句，直到遇到右大括号。
        while !self.is_current(Token::RightBrace) {
            statement_list.push(self.parse_statement()?);
        }

        self.expect(Token::RightBrace)?;
        Ok(Block { statement_list })
    }

    // 解析语句：根据第一个 token 判断是哪种语句。
    fn parse_statement(&mut self) -> CompileResult<Statement> {
        // 这里按"首 token"分流，可以减少回溯，报错位置也更直观。
        match self.current_token() {
            Token::Int | Token::Float | Token::Bool => self.parse_declare_statement(),
            Token::Name(_) => self.parse_assign_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::Return => self.parse_return_statement(),
            token => Err(format!("unexpected statement token: {token:?}")),
        }
    }

    // 解析变量声明：int x;
    fn parse_declare_statement(&mut self) -> CompileResult<Statement> {
        let type_name = self.parse_type_name()?;
        let name = self.parse_name()?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::DeclareVariable { type_name, name })
    }

    // 解析赋值语句：x = 123;
    fn parse_assign_statement(&mut self) -> CompileResult<Statement> {
        let name = self.parse_name()?;
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::AssignVariable { name, value })
    }

    // 解析 if 语句：if (条件) { ... } else { ... }
    fn parse_if_statement(&mut self) -> CompileResult<Statement> {
        self.expect(Token::If)?;
        self.expect(Token::LeftParen)?;

        let condition = self.parse_expression()?;

        self.expect(Token::RightParen)?;
        let then_block = self.parse_block()?;

        // else 部分可以省略，所以用 Option。
        let else_block = if self.is_current(Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::IfElse {
            condition,
            then_block,
            else_block,
        })
    }

    // 解析 while 循环：while (条件) { ... }
    fn parse_while_statement(&mut self) -> CompileResult<Statement> {
        self.expect(Token::While)?;
        self.expect(Token::LeftParen)?;

        let condition = self.parse_expression()?;

        self.expect(Token::RightParen)?;
        let body_block = self.parse_block()?;

        Ok(Statement::WhileLoop {
            condition,
            body_block,
        })
    }

    // 解析 return 语句：return 表达式;
    fn parse_return_statement(&mut self) -> CompileResult<Statement> {
        self.expect(Token::Return)?;

        let value = self.parse_expression()?;

        self.expect(Token::Semicolon)?;
        Ok(Statement::ReturnValue { value })
    }

    // 解析表达式：从比较运算开始（优先级最低）。
    fn parse_expression(&mut self) -> CompileResult<Expression> {
        self.parse_compare_expression()
    }

    // 解析比较表达式：a < b, a > b, a == b
    // 为什么比较运算优先级低于加减乘除？因为 a + b < c 应该先算 a+b 再比较。
    fn parse_compare_expression(&mut self) -> CompileResult<Expression> {
        let mut left = self.parse_add_expression()?;

        loop {
            let operator = match self.current_token() {
                Token::LessThan => BinaryOperator::LessThan,
                Token::GreaterThan => BinaryOperator::GreaterThan,
                Token::EqualEqual => BinaryOperator::Equal,
                _ => break,
            };

            self.advance();
            let right = self.parse_add_expression()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 解析加减表达式：a + b, a - b
    // 为什么加减优先级低于乘除？因为 a * b + c 应该先算 a*b 再加 c。
    fn parse_add_expression(&mut self) -> CompileResult<Expression> {
        let mut left = self.parse_mul_expression()?;

        loop {
            let operator = match self.current_token() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_mul_expression()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 解析乘除表达式：a * b, a / b
    // 为什么乘除优先级最高？因为 a * b / c 应该从左到右依次计算。
    fn parse_mul_expression(&mut self) -> CompileResult<Expression> {
        let mut left = self.parse_factor()?;

        loop {
            let operator = match self.current_token() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                _ => break,
            };

            self.advance();
            let right = self.parse_factor()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 解析最小表达式单元：变量名、数字、布尔值、或括号表达式。
    fn parse_factor(&mut self) -> CompileResult<Expression> {
        match self.current_token() {
            // 变量名：x
            Token::Name(_) => Ok(Expression::Variable(self.parse_name()?)),
            // 整数：123
            Token::Integer(value) => {
                let result = *value;
                self.advance();
                Ok(Expression::Integer(result))
            }
            // 浮点数：3.14
            Token::FloatValue(value) => {
                let result = *value;
                self.advance();
                Ok(Expression::Float(result))
            }
            // true
            Token::True => {
                self.advance();
                Ok(Expression::Bool(true))
            }
            // false
            Token::False => {
                self.advance();
                Ok(Expression::Bool(false))
            }
            // 括号表达式：(a + b)
            Token::LeftParen => {
                self.advance();
                let expression = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expression)
            }
            token => Err(format!("unexpected factor token: {token:?}")),
        }
    }

    // 解析类型名：int, float, bool
    fn parse_type_name(&mut self) -> CompileResult<TypeName> {
        let type_name = match self.current_token() {
            Token::Int => TypeName::Int,
            Token::Float => TypeName::Float,
            Token::Bool => TypeName::Bool,
            token => return Err(format!("expected type token, got: {token:?}")),
        };

        self.advance();
        Ok(type_name)
    }

    // 解析名字（变量名或函数名）。
    fn parse_name(&mut self) -> CompileResult<String> {
        match self.current_token() {
            Token::Name(name) => {
                let result = name.clone();
                self.advance();
                Ok(result)
            }
            token => Err(format!("expected name token, got: {token:?}")),
        }
    }

    // 期望当前 token 是指定的类型，否则报错。
    fn expect(&mut self, expected: Token) -> CompileResult<()> {
        if self.current_token() != &expected {
            return Err(format!("expected {expected:?}, got {:?}", self.current_token()));
        }

        self.advance();
        Ok(())
    }

    // 检查当前 token 是否是指定类型。
    fn is_current(&self, token: Token) -> bool {
        self.current_token() == &token
    }

    // 获取当前 token。
    fn current_token(&self) -> &Token {
        &self.token_list[self.current_index]
    }

    // 前进到下一个 token。
    fn advance(&mut self) {
        self.current_index += 1;
    }
}
