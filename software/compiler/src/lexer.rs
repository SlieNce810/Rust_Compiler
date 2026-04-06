// 词法分析器：把源代码文本拆成一个一个的 token（单词）。
// 比如把 "int x = 123;" 拆成 [Int, Name("x"), Assign, Integer(123), Semicolon]。

use crate::error::CompileResult;

// Token = 词法分析的最小单位。每个 token 代表源代码里的一个"词"。
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 关键字：有特殊含义的单词。
    Func,
    Int,
    Float,
    Bool,
    If,
    Else,
    While,
    Return,
    True,
    False,
    // 标识符：变量名、函数名等。
    Name(String),
    // 字面量：直接写在代码里的值。
    Integer(i64),
    FloatValue(f64),
    // 符号：各种标点和运算符。
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    LessThan,
    GreaterThan,
    EqualEqual,
    // 文件结束标记：帮助 parser 知道什么时候该停止。
    EndOfFile,
}

// 主入口：把整个源代码文本转成 token 列表。
pub fn tokenize(source_text: &str) -> CompileResult<Vec<Token>> {
    let mut char_stream = source_text.chars().peekable();
    let mut token_list = Vec::new();

    while let Some(&current_char) = char_stream.peek() {
        // BOM（Byte Order Mark）是某些编辑器在文件开头自动加的隐藏字符。
        // 这里忽略它，避免新手遇到"看不到的错误字符"。
        if current_char == '\u{feff}' {
            char_stream.next();
            continue;
        }

        // 空白字符（空格、换行、制表符）在语法里没有意义，直接跳过即可。
        if current_char.is_whitespace() {
            char_stream.next();
            continue;
        }

        // 字母或下划线开头 = 关键字或变量名。
        if current_char.is_ascii_alphabetic() || current_char == '_' {
            token_list.push(read_word_token(&mut char_stream));
            continue;
        }

        // 数字开头 = 整数或浮点数。
        if current_char.is_ascii_digit() {
            token_list.push(read_number_token(&mut char_stream)?);
            continue;
        }

        // 其他情况 = 符号（括号、运算符等）。
        token_list.push(read_symbol_token(&mut char_stream)?);
    }

    // 在最后加一个 EndOfFile，让 parser 知道什么时候该停下来。
    token_list.push(Token::EndOfFile);
    Ok(token_list)
}

// 读一个单词（关键字或变量名）。
fn read_word_token(char_stream: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Token {
    let mut word = String::new();

    // 一直读字母、数字、下划线，直到遇到其他字符。
    while let Some(&current_char) = char_stream.peek() {
        if current_char.is_ascii_alphanumeric() || current_char == '_' {
            word.push(current_char);
            char_stream.next();
            continue;
        }
        break;
    }

    // 先看是不是关键字，如果不是就当变量名处理。
    match word.as_str() {
        "func" => Token::Func,
        "int" => Token::Int,
        "float" => Token::Float,
        "bool" => Token::Bool,
        "if" => Token::If,
        "else" => Token::Else,
        "while" => Token::While,
        "return" => Token::Return,
        "true" => Token::True,
        "false" => Token::False,
        _ => Token::Name(word),
    }
}

// 读一个数字（整数或浮点数）。
fn read_number_token(char_stream: &mut std::iter::Peekable<std::str::Chars<'_>>) -> CompileResult<Token> {
    let mut number_text = String::new();
    let mut has_dot = false;

    while let Some(&current_char) = char_stream.peek() {
        if current_char.is_ascii_digit() {
            number_text.push(current_char);
            char_stream.next();
            continue;
        }

        // 遇到小数点：只允许出现一次，第二次小数点会在后续语法阶段报错。
        // 比如 "3.14.15" 会被拆成 FloatValue(3.14) 和错误。
        if current_char == '.' && !has_dot {
            has_dot = true;
            number_text.push(current_char);
            char_stream.next();
            continue;
        }

        break;
    }

    // 有小数点 = 浮点数，没有 = 整数。
    if has_dot {
        let value = number_text
            .parse::<f64>()
            .map_err(|_| format!("invalid float literal: {number_text}"))?;
        return Ok(Token::FloatValue(value));
    }

    let value = number_text
        .parse::<i64>()
        .map_err(|_| format!("invalid int literal: {number_text}"))?;
    Ok(Token::Integer(value))
}

// 读一个符号（括号、运算符等）。
fn read_symbol_token(char_stream: &mut std::iter::Peekable<std::str::Chars<'_>>) -> CompileResult<Token> {
    let Some(current_char) = char_stream.next() else {
        return Err("unexpected end while reading symbol".to_string());
    };

    let token = match current_char {
        '(' => Token::LeftParen,
        ')' => Token::RightParen,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ';' => Token::Semicolon,
        ',' => Token::Comma,
        '+' => Token::Plus,
        '-' => Token::Minus,
        '*' => Token::Star,
        '/' => Token::Slash,
        '<' => Token::LessThan,
        '>' => Token::GreaterThan,
        // = 可能是赋值，也可能是相等比较，需要看下一个字符。
        '=' => read_equal_or_assign(char_stream),
        _ => return Err(format!("unexpected char: {current_char}")),
    };

    Ok(token)
}

// 处理 = 和 ==：需要看下一个字符才能决定是赋值还是比较。
fn read_equal_or_assign(char_stream: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Token {
    // 如果下一个字符也是 =，那就是 ==（相等比较）。
    if char_stream.peek() == Some(&'=') {
        char_stream.next();
        return Token::EqualEqual;
    }
    // 否则是单个 =（赋值）。
    Token::Assign
}
