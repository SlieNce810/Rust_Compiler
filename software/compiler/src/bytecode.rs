use std::collections::HashMap;

use crate::ir::{ThreeAddressFunction, ThreeAddressProgram};

pub const MAGIC: &[u8; 4] = b"HBC1";
pub const VERSION: u8 = 1;
pub const MAX_REG_COUNT: usize = 32;

pub const OP_NOP: u8 = 0x00;
pub const OP_MOV: u8 = 0x01;
pub const OP_ADD: u8 = 0x02;
pub const OP_SUB: u8 = 0x03;
pub const OP_MUL: u8 = 0x04;
pub const OP_DIV: u8 = 0x05;
pub const OP_CMP_EQ: u8 = 0x06;
pub const OP_CMP_GT: u8 = 0x07;
pub const OP_CMP_LT: u8 = 0x08;
pub const OP_JMP: u8 = 0x09;
pub const OP_JMP_IF_ZERO: u8 = 0x0A;
pub const OP_RETURN: u8 = 0x0B;
pub const OP_HALT: u8 = 0x0C;

const MODE_LHS_IMM: u8 = 0x01;
const MODE_RHS_IMM: u8 = 0x02;

const INSTR_SIZE: usize = 12;

#[derive(Clone, Copy)]
struct EncodedInst {
    opcode: u8,
    dst: u8,
    lhs: u8,
    rhs: u8,
    mode: u8,
    reserved: u8,
    target: u16,
    imm: i32,
}

impl EncodedInst {
    fn to_bytes(self) -> [u8; INSTR_SIZE] {
        let mut out = [0u8; INSTR_SIZE];
        out[0] = self.opcode;
        out[1] = self.dst;
        out[2] = self.lhs;
        out[3] = self.rhs;
        out[4] = self.mode;
        out[5] = self.reserved;
        out[6..8].copy_from_slice(&self.target.to_le_bytes());
        out[8..12].copy_from_slice(&self.imm.to_le_bytes());
        out
    }
}

pub fn build_bytecode(program: &ThreeAddressProgram) -> Result<Vec<u8>, String> {
    ensure_supported_program(program)?;

    let function = &program.function_list[0];
    let (symbol_to_reg, reg_to_symbol) = collect_symbols(function)?;
    let label_to_pc = collect_labels(function)?;
    let instructions = encode_instructions(function, &symbol_to_reg, &label_to_pc)?;

    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.push(symbol_to_reg.len() as u8);
    write_u16(&mut out, reg_to_symbol.len(), "symbol count")?;
    write_u16(&mut out, instructions.len(), "instruction count")?;

    for (reg, name) in reg_to_symbol.iter().enumerate() {
        out.push(reg as u8);
        write_string(&mut out, name, "symbol name")?;
    }

    for inst in instructions {
        out.extend_from_slice(&inst.to_bytes());
    }

    Ok(out)
}

fn ensure_supported_program(program: &ThreeAddressProgram) -> Result<(), String> {
    if program.function_list.len() != 1 {
        return Err("bytecode v1 subset: only a single function is supported".to_string());
    }

    let function = &program.function_list[0];
    if function.name != "main" {
        return Err("bytecode v1 subset: only function main is supported".to_string());
    }

    for line in &function.line_list {
        ensure_supported_ir_line(line)?;
        if let Some((_, right)) = line.split_once(" = ") {
            for token in right.split_whitespace() {
                if is_float_literal(token) {
                    return Err("bytecode v1 does not support float type".to_string());
                }
            }
        }
    }

    Ok(())
}

fn ensure_supported_ir_line(line: &str) -> Result<(), String> {
    if line.starts_with("label ") {
        return Ok(());
    }
    if line.starts_with("goto ") {
        let label = line.trim_start_matches("goto ").trim();
        if label.starts_with('L') {
            return Ok(());
        }
        return Err(format!(
            "bytecode v1 subset: goto target must be label, got: {line}"
        ));
    }
    if let Some(rest) = line.strip_prefix("ifz ") {
        let parts = rest.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 3 || parts[1] != "goto" {
            return Err(format!("bytecode v1 subset: invalid ifz format: {line}"));
        }
        if is_number(parts[0]) {
            return Err(format!(
                "bytecode v1 subset: ifz expects register/variable, got immediate: {}",
                parts[0]
            ));
        }
        if !parts[2].starts_with('L') {
            return Err(format!(
                "bytecode v1 subset: ifz target must be label, got: {}",
                parts[2]
            ));
        }
        return Ok(());
    }
    if let Some(value) = line.strip_prefix("return ") {
        if value.trim().is_empty() {
            return Err("bytecode v1 subset: return requires a value".to_string());
        }
        return Ok(());
    }
    if let Some((left, right)) = line.split_once(" = ") {
        if left.trim().is_empty() || right.trim().is_empty() {
            return Err(format!("bytecode v1 subset: invalid assignment: {line}"));
        }
        return Ok(());
    }
    Err(format!(
        "bytecode v1 subset: unsupported IR statement: {line}"
    ))
}

fn is_float_literal(token: &str) -> bool {
    // Float literals contain a dot and are not an operator
    if !token.contains('.') {
        return false;
    }
    // Operators like == don't count
    if is_keyword_token(token) {
        return false;
    }
    token.parse::<f64>().is_ok()
}

fn collect_symbols(function: &ThreeAddressFunction) -> Result<(HashMap<String, u8>, Vec<String>), String> {
    let mut symbol_to_reg = HashMap::new();
    let mut reg_to_symbol = Vec::new();

    for line in &function.line_list {
        if let Some((left, right)) = line.split_once(" = ") {
            add_symbol_if_needed(left.trim(), &mut symbol_to_reg, &mut reg_to_symbol)?;
            for token in right.split_whitespace() {
                add_symbol_if_needed(token, &mut symbol_to_reg, &mut reg_to_symbol)?;
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("ifz ") {
            let parts = value.split_whitespace().collect::<Vec<_>>();
            if parts.len() == 3 {
                add_symbol_if_needed(parts[0], &mut symbol_to_reg, &mut reg_to_symbol)?;
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("return ") {
            add_symbol_if_needed(value.trim(), &mut symbol_to_reg, &mut reg_to_symbol)?;
        }
    }

    Ok((symbol_to_reg, reg_to_symbol))
}

fn add_symbol_if_needed(
    token: &str,
    symbol_to_reg: &mut HashMap<String, u8>,
    reg_to_symbol: &mut Vec<String>,
) -> Result<(), String> {
    if is_number(token) || is_keyword_token(token) {
        return Ok(());
    }

    if token.starts_with('L') {
        return Ok(());
    }

    if symbol_to_reg.contains_key(token) {
        return Ok(());
    }

    if reg_to_symbol.len() >= MAX_REG_COUNT {
        return Err("bytecode v1 register count exceeds 32".to_string());
    }

    let reg = reg_to_symbol.len() as u8;
    symbol_to_reg.insert(token.to_string(), reg);
    reg_to_symbol.push(token.to_string());
    Ok(())
}

fn collect_labels(function: &ThreeAddressFunction) -> Result<HashMap<String, u16>, String> {
    let mut map = HashMap::new();
    let mut pc = 0usize;

    for line in &function.line_list {
        if let Some(label) = line.strip_prefix("label ") {
            if map.insert(label.to_string(), pc as u16).is_some() {
                return Err(format!("duplicate label: {label}"));
            }
            continue;
        }

        pc += 1;
    }

    Ok(map)
}

fn encode_instructions(
    function: &ThreeAddressFunction,
    symbols: &HashMap<String, u8>,
    labels: &HashMap<String, u16>,
) -> Result<Vec<EncodedInst>, String> {
    let mut insts = Vec::new();

    for line in &function.line_list {
        if line.starts_with("label ") {
            continue;
        }

        let inst = if let Some(label) = line.strip_prefix("goto ") {
            encode_goto(label.trim(), labels)?
        } else if let Some(rest) = line.strip_prefix("ifz ") {
            encode_ifz(rest, symbols, labels)?
        } else if let Some(value) = line.strip_prefix("return ") {
            encode_return(value.trim(), symbols)?
        } else if let Some((left, right)) = line.split_once(" = ") {
            encode_assign(left.trim(), right.trim(), symbols)?
        } else {
            return Err(format!("unsupported IR line: {line}"));
        };

        insts.push(inst);
    }

    insts.push(EncodedInst {
        opcode: OP_HALT,
        dst: 0,
        lhs: 0,
        rhs: 0,
        mode: 0,
        reserved: 0,
        target: 0,
        imm: 0,
    });

    Ok(insts)
}

fn encode_assign(left: &str, right: &str, symbols: &HashMap<String, u8>) -> Result<EncodedInst, String> {
    let dst = reg_of(left, symbols)?;
    let parts = right.split_whitespace().collect::<Vec<_>>();

    if parts.len() == 1 {
        if let Some(value) = parse_i32(parts[0]) {
            return Ok(EncodedInst {
                opcode: OP_MOV,
                dst,
                lhs: 0,
                rhs: 0,
                mode: MODE_RHS_IMM,
                reserved: 0,
                target: 0,
                imm: value,
            });
        }

        return Ok(EncodedInst {
            opcode: OP_MOV,
            dst,
            lhs: reg_of(parts[0], symbols)?,
            rhs: 0,
            mode: 0,
            reserved: 0,
            target: 0,
            imm: 0,
        });
    }

    if parts.len() != 3 {
        return Err(format!("unsupported assignment expression: {right}"));
    }

    let opcode = match parts[1] {
        "+" => OP_ADD,
        "-" => OP_SUB,
        "*" => OP_MUL,
        "/" => OP_DIV,
        "==" => OP_CMP_EQ,
        ">" => OP_CMP_GT,
        "<" => OP_CMP_LT,
        _ => return Err(format!("unsupported binary operator: {}", parts[1])),
    };

    let lhs_num = parse_i32(parts[0]);
    let rhs_num = parse_i32(parts[2]);

    if lhs_num.is_some() && rhs_num.is_some() {
        let folded = fold_binary(opcode, lhs_num.unwrap_or(0), rhs_num.unwrap_or(0))?;
        return Ok(EncodedInst {
            opcode: OP_MOV,
            dst,
            lhs: 0,
            rhs: 0,
            mode: MODE_RHS_IMM,
            reserved: 0,
            target: 0,
            imm: folded,
        });
    }

    let mut mode = 0u8;
    let mut lhs = 0u8;
    let mut rhs = 0u8;
    let mut imm = 0i32;

    if let Some(num) = lhs_num {
        mode |= MODE_LHS_IMM;
        imm = num;
    } else {
        lhs = reg_of(parts[0], symbols)?;
    }

    if let Some(num) = rhs_num {
        if mode != 0 {
            return Err("binary expression with two immediates is not expected here".to_string());
        }
        mode |= MODE_RHS_IMM;
        imm = num;
    } else {
        rhs = reg_of(parts[2], symbols)?;
    }

    Ok(EncodedInst {
        opcode,
        dst,
        lhs,
        rhs,
        mode,
        reserved: 0,
        target: 0,
        imm,
    })
}

fn encode_goto(label: &str, labels: &HashMap<String, u16>) -> Result<EncodedInst, String> {
    let target = *labels
        .get(label)
        .ok_or_else(|| format!("undefined label in goto: {label}"))?;

    Ok(EncodedInst {
        opcode: OP_JMP,
        dst: 0,
        lhs: 0,
        rhs: 0,
        mode: 0,
        reserved: 0,
        target,
        imm: 0,
    })
}

fn encode_ifz(rest: &str, symbols: &HashMap<String, u8>, labels: &HashMap<String, u16>) -> Result<EncodedInst, String> {
    let parts = rest.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 || parts[1] != "goto" {
        return Err(format!("unsupported ifz line: ifz {rest}"));
    }

    let target = *labels
        .get(parts[2])
        .ok_or_else(|| format!("undefined label in ifz: {}", parts[2]))?;

    Ok(EncodedInst {
        opcode: OP_JMP_IF_ZERO,
        dst: 0,
        lhs: reg_of(parts[0], symbols)?,
        rhs: 0,
        mode: 0,
        reserved: 0,
        target,
        imm: 0,
    })
}

fn encode_return(value: &str, symbols: &HashMap<String, u8>) -> Result<EncodedInst, String> {
    if let Some(num) = parse_i32(value) {
        return Ok(EncodedInst {
            opcode: OP_RETURN,
            dst: 0,
            lhs: 0,
            rhs: 0,
            mode: MODE_LHS_IMM,
            reserved: 0,
            target: 0,
            imm: num,
        });
    }

    Ok(EncodedInst {
        opcode: OP_RETURN,
        dst: 0,
        lhs: reg_of(value, symbols)?,
        rhs: 0,
        mode: 0,
        reserved: 0,
        target: 0,
        imm: 0,
    })
}

fn fold_binary(opcode: u8, left: i32, right: i32) -> Result<i32, String> {
    let value = match opcode {
        OP_ADD => left.wrapping_add(right),
        OP_SUB => left.wrapping_sub(right),
        OP_MUL => left.wrapping_mul(right),
        OP_DIV => {
            if right == 0 {
                return Err("division by zero in constant expression".to_string());
            }
            left / right
        }
        OP_CMP_EQ => (left == right) as i32,
        OP_CMP_GT => (left > right) as i32,
        OP_CMP_LT => (left < right) as i32,
        _ => return Err("unsupported constant fold opcode".to_string()),
    };
    Ok(value)
}

fn reg_of(name: &str, symbols: &HashMap<String, u8>) -> Result<u8, String> {
    symbols
        .get(name)
        .copied()
        .ok_or_else(|| format!("unknown symbol: {name}"))
}

fn is_number(token: &str) -> bool {
    parse_i32(token).is_some()
}

fn parse_i32(token: &str) -> Option<i32> {
    token.parse::<i32>().ok()
}

fn is_keyword_token(token: &str) -> bool {
    matches!(token, "+" | "-" | "*" | "/" | "==" | ">" | "<" | "goto")
}

fn write_u16(out: &mut Vec<u8>, value: usize, field_name: &str) -> Result<(), String> {
    if value > u16::MAX as usize {
        return Err(format!("{field_name} exceeds u16 range"));
    }
    out.extend_from_slice(&(value as u16).to_le_bytes());
    Ok(())
}

fn write_string(out: &mut Vec<u8>, text: &str, field_name: &str) -> Result<(), String> {
    let bytes = text.as_bytes();
    write_u16(out, bytes.len(), field_name)?;
    out.extend_from_slice(bytes);
    Ok(())
}
