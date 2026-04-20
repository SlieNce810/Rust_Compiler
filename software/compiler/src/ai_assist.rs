use std::env;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub enum AiProviderKind {
    Mock,
    Local,
    DeepSeek,
}

impl AiProviderKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "mock" => Ok(Self::Mock),
            "local" => Ok(Self::Local),
            "cloud" => Ok(Self::DeepSeek),
            "deepseek" => Ok(Self::DeepSeek),
            _ => Err(format!(
                "unknown ai provider: {value}, expected mock|local|deepseek"
            )),
        }
    }
}

pub struct AiReport {
    pub markdown: String,
}

pub struct ErrorExplainInput {
    pub source_path: String,
    pub target_name: String,
    pub source_excerpt: String,
    pub compile_error: String,
    pub provider: AiProviderKind,
    pub api_key_override: Option<String>,
}

pub struct IrExplainInput {
    pub source_path: String,
    pub target_name: String,
    pub source_excerpt: String,
    pub ir_excerpt: String,
    pub provider: AiProviderKind,
    pub api_key_override: Option<String>,
}

pub struct SuggestTestsInput {
    pub source_path: String,
    pub target_name: String,
    pub source_excerpt: String,
    pub ir_excerpt: String,
    pub provider: AiProviderKind,
    pub api_key_override: Option<String>,
}

pub fn explain_error(input: &ErrorExplainInput) -> Result<AiReport, String> {
    let provider_name = provider_name(input.provider);
    let mut lines = Vec::new();
    lines.push("# AI Compile Report".to_string());
    lines.push(String::new());
    lines.push("## Metadata".to_string());
    lines.push(format!("- Source: `{}`", input.source_path));
    lines.push(format!("- Target: `{}`", input.target_name));
    lines.push(format!("- Status: `failure`"));
    lines.push(format!("- Provider: `{provider_name}`"));
    lines.push(String::new());
    lines.push("## Compile Error".to_string());
    lines.push("```text".to_string());
    lines.push(input.compile_error.clone());
    lines.push("```".to_string());
    lines.push(String::new());
    lines.push("## AI Explanation".to_string());
    lines.push(render_error_explanation(input)?);
    lines.push(String::new());
    lines.push("## Source Excerpt".to_string());
    lines.push("```hopping".to_string());
    lines.push(input.source_excerpt.clone());
    lines.push("```".to_string());
    lines.push(String::new());
    lines.push("> 建议仅作为辅助，请以编译器原始报错为准。".to_string());

    Ok(AiReport {
        markdown: lines.join("\n"),
    })
}

pub fn explain_ir(input: &IrExplainInput) -> Result<AiReport, String> {
    let provider_name = provider_name(input.provider);
    let mut lines = Vec::new();
    lines.push("# AI Compile Report".to_string());
    lines.push(String::new());
    lines.push("## Metadata".to_string());
    lines.push(format!("- Source: `{}`", input.source_path));
    lines.push(format!("- Target: `{}`", input.target_name));
    lines.push("- Status: `success`".to_string());
    lines.push(format!("- Provider: `{provider_name}`"));
    lines.push(String::new());
    lines.push("## IR Explanation".to_string());
    lines.push(render_ir_explanation(input)?);
    lines.push(String::new());
    lines.push("## Source Excerpt".to_string());
    lines.push("```hopping".to_string());
    lines.push(input.source_excerpt.clone());
    lines.push("```".to_string());
    lines.push(String::new());
    lines.push("## IR Excerpt".to_string());
    lines.push("```text".to_string());
    lines.push(input.ir_excerpt.clone());
    lines.push("```".to_string());
    lines.push(String::new());
    lines.push("> 建议仅作为辅助，请以编译器真实产物为准。".to_string());

    Ok(AiReport {
        markdown: lines.join("\n"),
    })
}

pub fn suggest_tests(input: &SuggestTestsInput) -> Result<AiReport, String> {
    let provider_name = provider_name(input.provider);
    let mut lines = Vec::new();
    lines.push("## Suggested Tests".to_string());
    lines.push(format!("_provider: `{provider_name}`_"));
    lines.push(String::new());
    lines.push(render_test_suggestions(input)?);
    Ok(AiReport {
        markdown: lines.join("\n"),
    })
}

fn render_error_explanation(input: &ErrorExplainInput) -> Result<String, String> {
    match input.provider {
        AiProviderKind::Mock => Ok(mock_error_explanation(&input.compile_error)),
        AiProviderKind::Local => match try_local_provider("error explain") {
            Ok(text) => Ok(text),
            Err(error) => Ok(format!(
                "- 本地 AI 不可用，已降级到规则解释。\n- 原因：{error}\n- 建议：先修复首个报错，再重跑编译。\n- 规则解释：{}",
                mock_error_explanation(&input.compile_error)
            )),
        },
        AiProviderKind::DeepSeek => match deepseek_error_explanation(input) {
            Ok(text) => Ok(text),
            Err(error) => Ok(format!(
                "- DeepSeek AI 不可用，已降级到规则解释。\n- 原因：{error}\n- 建议：检查网络、`HOPPING_AI_API_KEY` 或 `DEEPSEEK_API_KEY` 后重试。\n- 规则解释：{}",
                mock_error_explanation(&input.compile_error)
            )),
        },
    }
}

fn render_ir_explanation(input: &IrExplainInput) -> Result<String, String> {
    match input.provider {
        AiProviderKind::Mock => Ok(mock_ir_explanation(&input.ir_excerpt)),
        AiProviderKind::Local => match try_local_provider("ir explain") {
            Ok(text) => Ok(text),
            Err(error) => Ok(format!(
                "- 本地 AI 不可用，已降级到规则解释。\n- 原因：{error}\n- 规则解释：{}",
                mock_ir_explanation(&input.ir_excerpt)
            )),
        },
        AiProviderKind::DeepSeek => match deepseek_ir_explanation(input) {
            Ok(text) => Ok(text),
            Err(error) => Ok(format!(
                "- DeepSeek AI 不可用，已降级到规则解释。\n- 原因：{error}\n- 规则解释：{}",
                mock_ir_explanation(&input.ir_excerpt)
            )),
        },
    }
}

fn render_test_suggestions(input: &SuggestTestsInput) -> Result<String, String> {
    let base = mock_test_suggestions(&input.ir_excerpt);
    match input.provider {
        AiProviderKind::Mock => Ok(base),
        AiProviderKind::Local => match try_local_provider("suggest tests") {
            Ok(text) => Ok(format!("{text}\n\n{base}")),
            Err(error) => Ok(format!("- 本地 AI 不可用，使用规则建议。\n- 原因：{error}\n\n{base}")),
        },
        AiProviderKind::DeepSeek => match deepseek_suggest_tests(input) {
            Ok(text) => Ok(format!("{text}\n\n{base}")),
            Err(error) => Ok(format!("- DeepSeek AI 不可用，使用规则建议。\n- 原因：{error}\n\n{base}")),
        },
    }
}

fn mock_error_explanation(compile_error: &str) -> String {
    let mut tips = Vec::new();
    tips.push(format!("- 原始错误：`{compile_error}`"));
    if compile_error.contains("unexpected") {
        tips.push("- 推测是词法/语法不匹配：检查关键字、括号、分号和拼写。".to_string());
    } else if compile_error.contains("redeclaration") {
        tips.push("- 推测是重复声明：同一作用域变量名需要唯一。".to_string());
    } else if compile_error.contains("type") {
        tips.push("- 推测是类型不兼容：确认 int/bool 运算与比较表达式。".to_string());
    } else {
        tips.push("- 建议从首个报错位置向前后各检查 3-5 行上下文。".to_string());
    }
    tips.push("- 最小修复策略：每次只改一处，再重新编译确认。".to_string());
    tips.join("\n")
}

fn mock_ir_explanation(ir_excerpt: &str) -> String {
    let line_count = ir_excerpt.lines().count();
    let mut tips = Vec::new();
    tips.push(format!("- 当前 IR 片段共 `{line_count}` 行。"));
    tips.push("- 赋值语句会映射为 `MOV` 或算术/比较指令。".to_string());
    if ir_excerpt.contains("ifz ") {
        tips.push("- 发现 `ifz`：表示条件寄存器为 0 时跳转。".to_string());
    }
    if ir_excerpt.contains("goto ") {
        tips.push("- 发现 `goto`：对应无条件跳转。".to_string());
    }
    if ir_excerpt.contains("return ") {
        tips.push("- 发现 `return`：会写入 `retval` 并停机。".to_string());
    }
    tips.join("\n")
}

fn mock_test_suggestions(ir_excerpt: &str) -> String {
    let mut cases = Vec::new();
    cases.push("- 用例1：最小 happy path，覆盖声明/赋值/return。".to_string());
    if ir_excerpt.contains('/') {
        cases.push("- 用例2：除零场景，确认编译阶段或运行阶段错误行为符合预期。".to_string());
    }
    if ir_excerpt.contains("ifz ") || ir_excerpt.contains("goto ") {
        cases.push("- 用例3：分支与循环边界，确认跳转目标与终止条件。".to_string());
    }
    cases.push("- 用例4：非法输入（未知符号/错拼关键字）验证报错质量。".to_string());
    cases.join("\n")
}

fn try_local_provider(task_name: &str) -> Result<String, String> {
    let Some(base_url) = env::var_os("HOPPING_AI_BASE_URL") else {
        return Err("missing env HOPPING_AI_BASE_URL".to_string());
    };
    Ok(format!(
        "- 本地 provider 占位响应：已接入环境 `{}`，任务 `{task_name}`。",
        base_url.to_string_lossy()
    ))
}

fn deepseek_error_explanation(input: &ErrorExplainInput) -> Result<String, String> {
    let prompt = format!(
        "你是 Hopping 编译器错误修复助手。请根据以下信息输出：\n1) 错误原因\n2) 最小修改建议（步骤化）\n3) 一段可直接替换的示例代码片段\n\n目标平台: {}\n编译错误:\n{}\n\n源码片段:\n{}",
        input.target_name, input.compile_error, input.source_excerpt
    );
    call_deepseek_chat("error-explain", &prompt, input.api_key_override.as_deref())
}

fn deepseek_ir_explanation(input: &IrExplainInput) -> Result<String, String> {
    let prompt = format!(
        "你是 Hopping IR 讲解助手。请用简洁条目解释这段源码如何映射到 IR，重点说明控制流和 return 语义。\n\n目标平台: {}\n源码片段:\n{}\n\nIR 片段:\n{}",
        input.target_name, input.source_excerpt, input.ir_excerpt
    );
    call_deepseek_chat("ir-explain", &prompt, input.api_key_override.as_deref())
}

fn deepseek_suggest_tests(input: &SuggestTestsInput) -> Result<String, String> {
    let prompt = format!(
        "你是 Hopping 测试设计助手。请基于源码与 IR 给出 4-6 条高价值测试用例，包含：输入、预期行为、覆盖点。\n\n源码文件: {}\n目标平台: {}\n源码片段:\n{}\n\nIR 片段:\n{}",
        input.source_path, input.target_name, input.source_excerpt, input.ir_excerpt
    );
    call_deepseek_chat("suggest-tests", &prompt, input.api_key_override.as_deref())
}

fn call_deepseek_chat(
    task_name: &str,
    prompt: &str,
    api_key_override: Option<&str>,
) -> Result<String, String> {
    let key_info = load_api_key(api_key_override)?;
    let base_url = env::var("HOPPING_AI_BASE_URL")
        .unwrap_or_else(|_| "https://api.deepseek.com".to_string());
    let model = env::var("HOPPING_AI_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());
    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| format!("build http client failed: {error}"))?;

    let body = DeepSeekChatRequest {
        model,
        temperature: 0.2,
        messages: vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: "You are a compiler assistant. Provide practical and concise guidance.".to_string(),
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: format!("task={task_name}\n\n{prompt}"),
            },
        ],
    };

    let response = client
        .post(endpoint)
        .bearer_auth(&key_info.key)
        .json(&body)
        .send()
        .map_err(|error| format!("request deepseek failed: {error}"))?;

    let status = response.status();
    let text = response
        .text()
        .map_err(|error| format!("read deepseek response failed: {error}"))?;
    if !status.is_success() {
        return Err(format!("deepseek http {}: {}", status.as_u16(), truncate(&text, 400)));
    }

    let parsed: DeepSeekChatResponse = serde_json::from_str(&text)
        .map_err(|error| format!("parse deepseek response failed: {error}"))?;
    let content = parsed
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .unwrap_or_default();
    if content.is_empty() {
        return Err("deepseek response has empty content".to_string());
    }
    Ok(format!(
        "- 鉴权来源：`{}`（{}）\n\n{}",
        key_info.source, key_info.masked, content
    ))
}

fn load_api_key(api_key_override: Option<&str>) -> Result<ApiKeyInfo, String> {
    if let Some(key) = api_key_override {
        if !key.trim().is_empty() {
            return Ok(ApiKeyInfo {
                key: key.to_string(),
                source: "cli",
                masked: mask_key(key),
            });
        }
    }
    if let Ok(key) = env::var("HOPPING_AI_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(ApiKeyInfo {
                key: key.clone(),
                source: "HOPPING_AI_API_KEY",
                masked: mask_key(&key),
            });
        }
    }
    if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(ApiKeyInfo {
                key: key.clone(),
                source: "DEEPSEEK_API_KEY",
                masked: mask_key(&key),
            });
        }
    }
    Err("missing --ai-api-key or env HOPPING_AI_API_KEY or DEEPSEEK_API_KEY".to_string())
}

fn mask_key(key: &str) -> String {
    let len = key.chars().count();
    if len <= 8 {
        return format!("len={len}, ****");
    }
    let prefix = key.chars().take(4).collect::<String>();
    let suffix = key.chars().skip(len - 3).collect::<String>();
    format!("{prefix}***{suffix}, len={len}")
}

fn truncate(text: &str, max_len: usize) -> String {
    let mut result = text.chars().take(max_len).collect::<String>();
    if text.chars().count() > max_len {
        result.push_str("...");
    }
    result
}

fn provider_name(provider: AiProviderKind) -> &'static str {
    match provider {
        AiProviderKind::Mock => "mock",
        AiProviderKind::Local => "local",
        AiProviderKind::DeepSeek => "deepseek",
    }
}

#[derive(Serialize)]
struct DeepSeekChatRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    temperature: f32,
}

#[derive(Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct DeepSeekChatResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekResponseMessage,
}

#[derive(Deserialize)]
struct DeepSeekResponseMessage {
    content: String,
}

struct ApiKeyInfo {
    key: String,
    source: &'static str,
    masked: String,
}
