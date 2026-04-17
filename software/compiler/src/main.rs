// 主程序：命令行入口，组织编译流程。
// 编译步骤：读源代码 -> 词法分析 -> 语法分析 -> 语义检查 -> 中间代码 -> 汇编/IR/字节码输出。

use std::fs;
use std::path::PathBuf;

mod ast;
mod backend;
mod bytecode;
mod error;
mod ir;
mod lexer;
mod parser;
mod semantic;

fn main() {
    if let Err(error) = run_compile_flow() {
        eprintln!("compile failed: {error}");
        std::process::exit(1);
    }
}

fn run_compile_flow() -> Result<(), String> {
    let command_args = std::env::args().skip(1).collect::<Vec<_>>();
    if command_args.is_empty() {
        return Err(get_usage_text());
    }

    let options = parse_command_options(&command_args)?;
    let source_text = read_source_file(&options.input_path)?;
    let compile_output = compile_source(&source_text, &options.target_name)?;

    save_text_file(&options.output_path, &compile_output.assembly_text)?;
    if let Some(path) = &options.ir_output_path {
        save_text_file(path, &compile_output.ir_text)?;
    }
    if let Some(path) = &options.bytecode_output_path {
        save_binary_file(path, &compile_output.bytecode)?;
    }
    if options.emit_demo_artifacts {
        save_text_file(&PathBuf::from("../examples/ir/main.ir"), &compile_output.ir_text)?;
        save_binary_file(
            &PathBuf::from("../examples/bytecode/main.hbc"),
            &compile_output.bytecode,
        )?;
    }

    println!(
        "compile success: {} -> {}",
        options.input_path.display(),
        options.output_path.display()
    );
    Ok(())
}

struct CommandOptions {
    input_path: PathBuf,
    output_path: PathBuf,
    target_name: String,
    ir_output_path: Option<PathBuf>,
    bytecode_output_path: Option<PathBuf>,
    emit_demo_artifacts: bool,
}

fn parse_command_options(command_args: &[String]) -> Result<CommandOptions, String> {
    let mut input_path: Option<PathBuf> = None;
    let mut output_path = PathBuf::from("out.asm");
    let mut target_name = String::from("stm32f403");
    let mut ir_output_path: Option<PathBuf> = None;
    let mut bytecode_output_path: Option<PathBuf> = None;
    let mut emit_demo_artifacts = false;

    let mut index = 0;
    while index < command_args.len() {
        index = parse_one_option(
            command_args,
            index,
            &mut input_path,
            &mut output_path,
            &mut target_name,
            &mut ir_output_path,
            &mut bytecode_output_path,
            &mut emit_demo_artifacts,
        )?;
    }

    let Some(input_path) = input_path else {
        return Err("missing input source file".to_string());
    };

    Ok(CommandOptions {
        input_path,
        output_path,
        target_name,
        ir_output_path,
        bytecode_output_path,
        emit_demo_artifacts,
    })
}

fn parse_one_option(
    command_args: &[String],
    index: usize,
    input_path: &mut Option<PathBuf>,
    output_path: &mut PathBuf,
    target_name: &mut String,
    ir_output_path: &mut Option<PathBuf>,
    bytecode_output_path: &mut Option<PathBuf>,
    emit_demo_artifacts: &mut bool,
) -> Result<usize, String> {
    let current = &command_args[index];

    if current == "-o" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing output path after -o".to_string());
        };
        *output_path = PathBuf::from(value);
        return Ok(index + 2);
    }

    if current == "--target" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing target name".to_string());
        };
        *target_name = value.to_string();
        return Ok(index + 2);
    }

    if current == "--emit-ir" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing output path after --emit-ir".to_string());
        };
        *ir_output_path = Some(PathBuf::from(value));
        return Ok(index + 2);
    }

    if current == "--emit-bytecode" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing output path after --emit-bytecode".to_string());
        };
        *bytecode_output_path = Some(PathBuf::from(value));
        return Ok(index + 2);
    }

    if current == "--emit-demo-artifacts" {
        *emit_demo_artifacts = true;
        return Ok(index + 1);
    }

    if current.starts_with('-') {
        return Err(format!("unknown arg: {current}"));
    }

    *input_path = Some(PathBuf::from(current));
    Ok(index + 1)
}

fn read_source_file(input_path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(input_path)
        .map_err(|error| format!("failed to read {}: {error}", input_path.display()))
}

fn compile_source(source_text: &str, target_name: &str) -> Result<CompileOutput, String> {
    let token_list = lexer::tokenize(source_text)?;
    let mut parser = parser::Parser::new(token_list);
    let program = parser.parse_program()?;

    semantic::check_program(&program)?;
    let three_address_program = ir::build_three_address_code(&program);

    let assembly_text = backend::build_assembly_code(&three_address_program, target_name);
    let ir_text = ir::build_ir_text(&three_address_program);
    let bytecode = bytecode::build_bytecode(&three_address_program)?;

    Ok(CompileOutput {
        assembly_text,
        ir_text,
        bytecode,
    })
}

fn save_text_file(output_path: &PathBuf, text: &str) -> Result<(), String> {
    fs::write(output_path, text)
        .map_err(|error| format!("failed to write {}: {error}", output_path.display()))
}

fn save_binary_file(output_path: &PathBuf, data: &[u8]) -> Result<(), String> {
    fs::write(output_path, data)
        .map_err(|error| format!("failed to write {}: {error}", output_path.display()))
}

struct CompileOutput {
    assembly_text: String,
    ir_text: String,
    bytecode: Vec<u8>,
}

fn get_usage_text() -> String {
    "usage: mini_embedded_compiler <input.hopping> [-o output.asm] [--target stm32f403|esp32] [--emit-ir out.ir] [--emit-bytecode out.hbc] [--emit-demo-artifacts]".to_string()
}
