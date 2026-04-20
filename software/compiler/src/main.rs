// 主程序：命令行入口，组织编译流程。
// 编译步骤：读源代码 -> 词法分析 -> 语法分析 -> 语义检查 -> 中间代码 -> 汇编/IR/字节码输出。

use std::fs;
use std::path::PathBuf;

mod ai_assist;
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
    let compile_output = match compile_source(&source_text, &options.target_name) {
        Ok(output) => output,
        Err(compile_error) => {
            print_failure_ai_guidance(&options, &source_text, &compile_error);
            return Err(compile_error);
        }
    };

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
    if !options.ai_only_on_error {
        write_success_ai_report(&options, &source_text, &compile_output.ir_text)?;
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
    ai_explain_error: bool,
    ai_only_on_error: bool,
    ai_report_path: Option<PathBuf>,
    ai_api_key: Option<String>,
    ai_provider: ai_assist::AiProviderKind,
}

fn parse_command_options(command_args: &[String]) -> Result<CommandOptions, String> {
    let mut input_path: Option<PathBuf> = None;
    let mut output_path = PathBuf::from("out.asm");
    let mut target_name = String::from("stm32f403");
    let mut ir_output_path: Option<PathBuf> = None;
    let mut bytecode_output_path: Option<PathBuf> = None;
    let mut emit_demo_artifacts = false;
    let mut ai_explain_error = false;
    let mut ai_only_on_error = false;
    let mut ai_report_path: Option<PathBuf> = None;
    let mut ai_api_key: Option<String> = None;
    let mut ai_provider = ai_assist::AiProviderKind::Mock;

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
            &mut ai_explain_error,
            &mut ai_only_on_error,
            &mut ai_report_path,
            &mut ai_api_key,
            &mut ai_provider,
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
        ai_explain_error,
        ai_only_on_error,
        ai_report_path,
        ai_api_key,
        ai_provider,
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
    ai_explain_error: &mut bool,
    ai_only_on_error: &mut bool,
    ai_report_path: &mut Option<PathBuf>,
    ai_api_key: &mut Option<String>,
    ai_provider: &mut ai_assist::AiProviderKind,
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

    if current == "--ai-explain-error" {
        *ai_explain_error = true;
        return Ok(index + 1);
    }

    if current == "--ai-only-on-error" {
        *ai_only_on_error = true;
        return Ok(index + 1);
    }

    if current == "--ai-report" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing output path after --ai-report".to_string());
        };
        *ai_report_path = Some(PathBuf::from(value));
        return Ok(index + 2);
    }

    if current == "--ai-provider" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing provider after --ai-provider".to_string());
        };
        *ai_provider = ai_assist::AiProviderKind::parse(value)?;
        return Ok(index + 2);
    }

    if current == "--ai-api-key" {
        let Some(value) = command_args.get(index + 1) else {
            return Err("missing api key after --ai-api-key".to_string());
        };
        *ai_api_key = Some(value.to_string());
        return Ok(index + 2);
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

fn print_failure_ai_guidance(
    options: &CommandOptions,
    source_text: &str,
    compile_error: &str,
) {
    if !options.ai_explain_error {
        return;
    }

    let request = ai_assist::ErrorExplainInput {
        source_path: options.input_path.to_string_lossy().to_string(),
        target_name: options.target_name.clone(),
        source_excerpt: build_source_excerpt(source_text, 120),
        compile_error: compile_error.to_string(),
        provider: options.ai_provider,
        api_key_override: options.ai_api_key.clone(),
    };
    match ai_assist::explain_error(&request) {
        Ok(report) => eprintln!("\n===== AI Repair Guidance =====\n{}\n", report.markdown),
        Err(error) => eprintln!(
            "\n===== AI Repair Guidance =====\nAI assist unavailable: {error}\n"
        ),
    }
}

fn write_success_ai_report(
    options: &CommandOptions,
    source_text: &str,
    ir_text: &str,
) -> Result<(), String> {
    let Some(report_path) = &options.ai_report_path else {
        return Ok(());
    };

    let explain_request = ai_assist::IrExplainInput {
        source_path: options.input_path.to_string_lossy().to_string(),
        target_name: options.target_name.clone(),
        source_excerpt: build_source_excerpt(source_text, 120),
        ir_excerpt: build_source_excerpt(ir_text, 180),
        provider: options.ai_provider,
        api_key_override: options.ai_api_key.clone(),
    };
    let test_request = ai_assist::SuggestTestsInput {
        source_path: options.input_path.to_string_lossy().to_string(),
        target_name: options.target_name.clone(),
        source_excerpt: build_source_excerpt(source_text, 120),
        ir_excerpt: build_source_excerpt(ir_text, 180),
        provider: options.ai_provider,
        api_key_override: options.ai_api_key.clone(),
    };

    let mut sections = Vec::new();
    sections.push(ai_assist::explain_ir(&explain_request)?.markdown);
    sections.push(ai_assist::suggest_tests(&test_request)?.markdown);
    save_text_file(report_path, &sections.join("\n\n"))
}

fn build_source_excerpt(text: &str, max_lines: usize) -> String {
    text.lines().take(max_lines).collect::<Vec<_>>().join("\n")
}

fn get_usage_text() -> String {
    "usage: mini_embedded_compiler <input.hopping> [-o output.asm] [--target stm32f403|esp32] [--emit-ir out.ir] [--emit-bytecode out.hbc] [--emit-demo-artifacts] [--ai-explain-error] [--ai-only-on-error] [--ai-report out.md] [--ai-provider mock|local|deepseek] [--ai-api-key sk-xxxx]".to_string()
}
