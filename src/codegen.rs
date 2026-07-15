use crate::parser::{Program, Stmt, Expr, Op};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Str,
}

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub suggestion: String,
}

pub fn type_check(program: &Program) -> Result<HashMap<String, Type>, TypeError> {
    let mut symbol_table = HashMap::new();
    for stmt in &program.statements {
        type_check_stmt(stmt, &mut symbol_table)?;
    }
    Ok(symbol_table)
}

fn type_check_stmt(stmt: &Stmt, symbol_table: &mut HashMap<String, Type>) -> Result<(), TypeError> {
    match stmt {
        Stmt::Assign { name, value, name_line, name_col } => {
            let val_type = type_check_expr(value, symbol_table, *name_line, *name_col)?;
            if let Some(existing_type) = symbol_table.get(name) {
                if existing_type != &val_type {
                    return Err(TypeError {
                        message: format!(
                            "Type mismatch for variable '{}'. It was previously defined as a {:?}, but is now assigned a {:?}.",
                            name, existing_type, val_type
                        ),
                        line: *name_line,
                        column: *name_col,
                        suggestion: format!(
                            "Variables in BroLang have a fixed type. Create a new variable or ensure the assigned value is a {:?}.",
                            existing_type
                        ),
                    });
                }
            } else {
                symbol_table.insert(name.clone(), val_type);
            }
        }
        Stmt::Print(expr) => {
            type_check_expr(expr, symbol_table, 1, 1)?;
        }
        Stmt::If { cond, then_branch } => {
            let cond_type = type_check_expr(cond, symbol_table, 1, 1)?;
            if cond_type != Type::Int {
                return Err(TypeError {
                    message: "The condition of an 'if' / 'wenn' statement must evaluate to a number (0 for false, non-zero for true).".to_string(),
                    line: 1,
                    column: 1,
                    suggestion: "Ensure the condition is a comparison or an integer value.".to_string(),
                });
            }
            for s in then_branch {
                type_check_stmt(s, symbol_table)?;
            }
        }
        Stmt::While { cond, body } => {
            let cond_type = type_check_expr(cond, symbol_table, 1, 1)?;
            if cond_type != Type::Int {
                return Err(TypeError {
                    message: "The condition of a 'while' / 'solange' statement must evaluate to a number (0 for false, non-zero for true).".to_string(),
                    line: 1,
                    column: 1,
                    suggestion: "Ensure the condition is a comparison or an integer value.".to_string(),
                });
            }
            for s in body {
                type_check_stmt(s, symbol_table)?;
            }
        }
    }
    Ok(())
}

fn type_check_expr(
    expr: &Expr,
    symbol_table: &HashMap<String, Type>,
    line: usize,
    column: usize,
) -> Result<Type, TypeError> {
    match expr {
        Expr::Int(_) => Ok(Type::Int),
        Expr::Str(_) => Ok(Type::Str),
        Expr::Var(name) => {
            if let Some(t) = symbol_table.get(name) {
                Ok(t.clone())
            } else {
                Err(TypeError {
                    message: format!("Variable '{}' is used before being set.", name),
                    line,
                    column,
                    suggestion: format!("Initialize the variable first using 'set {} to [value]' (or 'setze {} auf [value]').", name, name),
                })
            }
        }
        Expr::Binary { op, left, right } => {
            let left_type = type_check_expr(left, symbol_table, line, column)?;
            let right_type = type_check_expr(right, symbol_table, line, column)?;
            match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div => {
                    if left_type == Type::Int && right_type == Type::Int {
                        Ok(Type::Int)
                    } else {
                        Err(TypeError {
                            message: format!("Arithmetic operator '{}' is only supported for numbers, but got {:?} and {:?}.", op.to_string_representation(), left_type, right_type),
                            line,
                            column,
                            suggestion: "Ensure both sides of the arithmetic operation are number values or variables.".to_string(),
                        })
                    }
                }
                Op::Eq | Op::NotEq | Op::Lt | Op::LtEq | Op::Gt | Op::GtEq => {
                    if left_type == right_type {
                        Ok(Type::Int)
                    } else {
                        Err(TypeError {
                            message: format!("Cannot compare different types: {:?} and {:?}.", left_type, right_type),
                            line,
                            column,
                            suggestion: "Ensure both sides of the comparison are of the same type.".to_string(),
                        })
                    }
                }
            }
        }
    }
}

pub fn print_type_error(err: &TypeError, source_lines: &[String]) {
    eprintln!("\x1b[1;31mType Error:\x1b[0m {}", err.message);
    if err.line > 0 && err.line <= source_lines.len() {
        eprintln!("At line {}, column {}:", err.line, err.column);
        eprintln!();
        let line_content = &source_lines[err.line - 1];
        eprintln!("  {:3} | {}", err.line, line_content);
        let padding = " ".repeat(err.column - 1);
        eprintln!("      | \x1b[1;31m{}^\x1b[0m", padding);
    }
    eprintln!();
    eprintln!("\x1b[1;32mSuggestion:\x1b[0m {}", err.suggestion);
}

fn collect_string_literals(program: &Program) -> Vec<String> {
    let mut literals = Vec::new();
    for stmt in &program.statements {
        collect_stmt_strings(stmt, &mut literals);
    }
    literals
}

fn collect_stmt_strings(stmt: &Stmt, literals: &mut Vec<String>) {
    match stmt {
        Stmt::Assign { value, .. } => collect_expr_strings(value, literals),
        Stmt::Print(expr) => collect_expr_strings(expr, literals),
        Stmt::If { cond, then_branch } => {
            collect_expr_strings(cond, literals);
            for s in then_branch {
                collect_stmt_strings(s, literals);
            }
        }
        Stmt::While { cond, body } => {
            collect_expr_strings(cond, literals);
            for s in body {
                collect_stmt_strings(s, literals);
            }
        }
    }
}

fn collect_expr_strings(expr: &Expr, literals: &mut Vec<String>) {
    match expr {
        Expr::Str(s) => {
            if !literals.contains(s) {
                literals.push(s.clone());
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_expr_strings(left, literals);
            collect_expr_strings(right, literals);
        }
        _ => {}
    }
}

fn escape_fasm_string(s: &str) -> String {
    s.replace("'", "''")
}

fn get_expr_type(expr: &Expr, symbol_table: &HashMap<String, Type>) -> Type {
    match expr {
        Expr::Int(_) => Type::Int,
        Expr::Str(_) => Type::Str,
        Expr::Var(name) => symbol_table.get(name).cloned().unwrap_or(Type::Int),
        Expr::Binary { op, .. } => {
            match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div => Type::Int,
                Op::Eq | Op::NotEq | Op::Lt | Op::LtEq | Op::Gt | Op::GtEq => Type::Int,
            }
        }
    }
}

pub fn generate_assembly(program: &Program, symbol_table: &HashMap<String, Type>) -> String {
    let mut asm = String::new();
    
    // Header
    asm.push_str("format PE64 console\n");
    asm.push_str("entry start\n\n");
    
    // String literals mapping
    let string_literals = collect_string_literals(program);
    let mut string_map = HashMap::new();
    for (idx, lit) in string_literals.iter().enumerate() {
        string_map.insert(lit.clone(), idx);
    }
    
    // Section .data
    asm.push_str("section '.data' data readable writeable\n");
    asm.push_str("  fmt_int db '%lld', 13, 10, 0\n");
    asm.push_str("  fmt_str db '%s', 13, 10, 0\n");
    
    for (lit, idx) in &string_map {
        let escaped = escape_fasm_string(lit);
        asm.push_str(&format!("  str_lit_{} db '{}', 0\n", idx, escaped));
    }
    
    for var_name in symbol_table.keys() {
        asm.push_str(&format!("  var_{} dq 0\n", var_name));
    }
    asm.push_str("\n");
    
    // Section .text
    asm.push_str("section '.text' code readable executable\n");
    asm.push_str("start:\n");
    asm.push_str("  sub rsp, 40\n\n"); // 32 shadow space + 8 alignment
    
    let mut label_counter = 0;
    for stmt in &program.statements {
        codegen_stmt(stmt, &mut asm, &string_map, symbol_table, &mut label_counter);
    }
    
    asm.push_str("  mov rcx, 0\n");
    asm.push_str("  call [ExitProcess]\n\n");
    
    // Section .idata
    asm.push_str("section '.idata' import data readable\n");
    asm.push_str("  library kernel32, 'KERNEL32.DLL', \\\n");
    asm.push_str("          msvcrt, 'MSVCRT.DLL'\n\n");
    asm.push_str("  import kernel32, \\\n");
    asm.push_str("         ExitProcess, 'ExitProcess'\n\n");
    asm.push_str("  import msvcrt, \\\n");
    asm.push_str("         printf, 'printf'\n");
    
    asm
}

fn codegen_stmt(
    stmt: &Stmt,
    asm: &mut String,
    string_map: &HashMap<String, usize>,
    symbol_table: &HashMap<String, Type>,
    label_counter: &mut usize,
) {
    match stmt {
        Stmt::Assign { name, value, .. } => {
            codegen_expr(value, asm, string_map);
            asm.push_str(&format!("  mov [var_{}], rax\n", name));
        }
        Stmt::Print(expr) => {
            codegen_expr(expr, asm, string_map);
            let expr_type = get_expr_type(expr, symbol_table);
            asm.push_str("  mov rdx, rax\n");
            match expr_type {
                Type::Int => {
                    asm.push_str("  mov rcx, fmt_int\n");
                }
                Type::Str => {
                    asm.push_str("  mov rcx, fmt_str\n");
                }
            }
            asm.push_str("  call [printf]\n");
        }
        Stmt::If { cond, then_branch } => {
            let label_idx = *label_counter;
            *label_counter += 1;
            codegen_expr(cond, asm, string_map);
            asm.push_str("  cmp rax, 0\n");
            asm.push_str(&format!("  je .L_end_{}\n", label_idx));
            for s in then_branch {
                codegen_stmt(s, asm, string_map, symbol_table, label_counter);
            }
            asm.push_str(&format!(".L_end_{}:\n", label_idx));
        }
        Stmt::While { cond, body } => {
            let label_idx = *label_counter;
            *label_counter += 1;
            asm.push_str(&format!(".L_cond_{}:\n", label_idx));
            codegen_expr(cond, asm, string_map);
            asm.push_str("  cmp rax, 0\n");
            asm.push_str(&format!("  je .L_end_{}\n", label_idx));
            for s in body {
                codegen_stmt(s, asm, string_map, symbol_table, label_counter);
            }
            asm.push_str(&format!("  jmp .L_cond_{}\n", label_idx));
            asm.push_str(&format!(".L_end_{}:\n", label_idx));
        }
    }
}

fn codegen_expr(expr: &Expr, asm: &mut String, string_map: &HashMap<String, usize>) {
    match expr {
        Expr::Int(val) => {
            asm.push_str(&format!("  mov rax, {}\n", val));
        }
        Expr::Str(val) => {
            let idx = string_map.get(val).unwrap();
            asm.push_str(&format!("  mov rax, str_lit_{}\n", idx));
        }
        Expr::Var(name) => {
            asm.push_str(&format!("  mov rax, [var_{}]\n", name));
        }
        Expr::Binary { op, left, right } => {
            codegen_expr(left, asm, string_map);
            asm.push_str("  push rax\n");
            codegen_expr(right, asm, string_map);
            asm.push_str("  pop r10\n"); // r10 contains left, rax contains right
            
            match op {
                Op::Add => {
                    asm.push_str("  add rax, r10\n");
                }
                Op::Sub => {
                    asm.push_str("  sub r10, rax\n");
                    asm.push_str("  mov rax, r10\n");
                }
                Op::Mul => {
                    asm.push_str("  imul rax, r10\n");
                }
                Op::Div => {
                    asm.push_str("  mov r11, rax\n"); // right
                    asm.push_str("  mov rax, r10\n"); // left
                    asm.push_str("  cqo\n");
                    asm.push_str("  idiv r11\n");
                }
                Op::Eq => {
                    asm.push_str("  cmp r10, rax\n");
                    asm.push_str("  sete al\n");
                    asm.push_str("  movzx rax, al\n");
                }
                Op::NotEq => {
                    asm.push_str("  cmp r10, rax\n");
                    asm.push_str("  setne al\n");
                    asm.push_str("  movzx rax, al\n");
                }
                Op::Lt => {
                    asm.push_str("  cmp r10, rax\n");
                    asm.push_str("  setl al\n");
                    asm.push_str("  movzx rax, al\n");
                }
                Op::LtEq => {
                    asm.push_str("  cmp r10, rax\n");
                    asm.push_str("  setle al\n");
                    asm.push_str("  movzx rax, al\n");
                }
                Op::Gt => {
                    asm.push_str("  cmp r10, rax\n");
                    asm.push_str("  setg al\n");
                    asm.push_str("  movzx rax, al\n");
                }
                Op::GtEq => {
                    asm.push_str("  cmp r10, rax\n");
                    asm.push_str("  setge al\n");
                    asm.push_str("  movzx rax, al\n");
                }
            }
        }
    }
}
