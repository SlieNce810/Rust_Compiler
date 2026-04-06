// 主程序：命令行入口，组织编译流程。
// 编译步骤：读源代码 -> 词法分析 -> 语法分析 -> 语义检查 -> 中间代码 -> 汇编代码。

use std::fs;
use std::path::PathBuf;

mod ast;
mod backend;
mod error;
mod ir;
mod lexer;
mod parser;
mod semantic;

// 程序入口：捕获错误并打印。
fn main() {
    if let Err(error) = run_compile_flow() {
        eprintln!("compile failed: {error}");
        std::process::exit(1);
    }
}

// 主编译流程：解析参数 -> 读文件 -> 编译 -> 写文件。
fn run_compile_flow() -> Result<(), String> {
    // 先拿命令行参数；没有参数时直接给用法，避免后面出现级联报错。
    let command_args = std::env::args().skip(1).collect::<Vec<_>>();
    if command_args.is_empty() {
        return Err(get_usage_text());
    }

    let options = parse_command_options(&command_args)?;

    // 这里故意拆成"读源代码 -> 编译 -> 写文件"三步，让新手能顺着排查问题。
    let source_text = read_source_file(&options.input_path)?;
    let assembly_text = compile_source_to_assembly(&source_text, &options.target_name)?;

    save_output_file(&options.output_path, &assembly_text)?;
    println!(
        "compile success: {} -> {}",
        options.input_path.display(),
        options.output_path.display()
    );
    Ok(())
}

// 命令行选项：输入文件、输出文件、目标平台。
struct CommandOptions {
    input_path: PathBuf,
    output_path: PathBuf,
    target_name: String,
}

// 解析命令行参数。
fn parse_command_options(command_args: &[String]) -> Result<CommandOptions, String> {
    let mut input_path: Option<PathBuf> = None;
    let mut output_path = PathBuf::from("out.asm"); // 默认输出文件名
    let mut target_name = String::from("stm32f403"); // 默认目标平台

    let mut index = 0;
    while index < command_args.len() {
        index = parse_one_option(
            command_args,
            index,
            &mut input_path,
            &mut output_path,
            &mut target_name,
        )?;
    }

    // 必须提供输入文件。
    let Some(input_path) = input_path else {
        return Err("missing input source file".to_string());
    };

    Ok(CommandOptions {
        input_path,
        output_path,
        target_name,
    })
}

// 解析单个参数，返回下一个要处理的索引。
fn parse_one_option(
    command_args: &[String],
    index: usize,
    input_path: &mut Option<PathBuf>,
    output_path: &mut PathBuf,
    target_name: &mut String,
) -> Result<usize, String> {
    let current = &command_args[index];

    // -o 输出文件名
    if current == "-o" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing output path after -o".to_string());
        };
        *output_path = PathBuf::from(value);
        return Ok(index + 2);
    }

    // --target 目标平台
    if current == "--target" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing target name".to_string());
        };
        *target_name = value.to_string();
        return Ok(index + 2);
    }

    // 以 - 开头但是不认识，报错。
    if current.starts_with('-') {
        return Err(format!("unknown arg: {current}"));
    }

    // 否则就是输入文件名。
    *input_path = Some(PathBuf::from(current));
    Ok(index + 1)
}

// 读取源代码文件。
fn read_source_file(input_path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(input_path)
        .map_err(|error| format!("failed to read {}: {error}", input_path.display()))
}

// 核心编译逻辑：把源代码编译成汇编代码。
fn compile_source_to_assembly(source_text: &str, target_name: &str) -> Result<String, String> {
    // 步骤 1：词法分析 - 把文本拆成 token。
    let token_list = lexer::tokenize(source_text)?;

    // 步骤 2：语法分析 - 把 token 变成语法树。
    let mut parser = parser::Parser::new(token_list);
    let program = parser.parse_program()?;

    // 步骤 3：语义检查 - 做类型和作用域检查，尽早暴露源代码错误。
    semantic::check_program(&program)?;

    // 步骤 4：生成中间代码 - 先转三地址码，再交给后端做目标平台文本生成。
    let three_address_program = ir::build_three_address_code(&program);

    // 步骤 5：生成汇编代码。
    Ok(backend::build_assembly_code(
        &three_address_program,
        target_name,
    ))
}

// 保存汇编代码到文件。
fn save_output_file(output_path: &PathBuf, assembly_text: &str) -> Result<(), String> {
    fs::write(output_path, assembly_text)
        .map_err(|error| format!("failed to write {}: {error}", output_path.display()))
}

// 获取用法提示文本。
fn get_usage_text() -> String {
    "usage: mini_embedded_compiler <input.mc> [-o output.asm] [--target stm32f403|esp32]"
        .to_string()
}
