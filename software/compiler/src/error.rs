// 错误处理：用 Result<T, String> 统一返回错误信息。
// 为什么不用复杂的错误类型？因为这个编译器面向教学，字符串错误信息足够清晰。
// 新手看到 "redeclaration of variable: x" 比 "CompileError::Redeclaration" 更容易理解。
pub type CompileResult<T> = Result<T, String>;
