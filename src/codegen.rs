use crate::parser::{Program, Stmt, Expr, Op};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Str,
    List(Box<Type>),
    Dict(HashMap<String, Type>),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Storage {
    Global,
    Local(i32), // Offset relative to RBP
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub var_type: Type,
    pub storage: Storage,
}

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub suggestion: String,
}

pub struct CodegenCtx {
    pub local_offsets: HashMap<String, i32>,
    pub func_name: String,
}

pub fn type_check(program: &Program) -> Result<HashMap<String, VariableInfo>, TypeError> {
    let mut scope_stack = vec![HashMap::new()]; // Start with global scope
    let mut function_signatures = HashMap::new();
    
    // Pre-scan functions to populate signatures
    for stmt in &program.statements {
        if let Stmt::FnDef { name, params, body } = stmt {
            let mut local_symbols = HashMap::new();
            for p in params {
                local_symbols.insert(p.clone(), VariableInfo { var_type: Type::Int, storage: Storage::Local(0) });
            }
            for s in body {
                pre_populate_locals(s, &mut local_symbols);
            }
            let body_symbols_stack = vec![HashMap::new(), local_symbols];
            let ret_type = infer_return_type(body, &body_symbols_stack);
            let param_types = vec![Type::Int; params.len()];
            function_signatures.insert(name.clone(), (param_types, ret_type));
        }
    }
    
    for stmt in &program.statements {
        type_check_stmt(stmt, &mut scope_stack, &function_signatures, None)?;
    }
    Ok(scope_stack.remove(0))
}

fn pre_populate_locals(stmt: &Stmt, local_symbols: &mut HashMap<String, VariableInfo>) {
    match stmt {
        Stmt::Assign { name, value, .. } => {
            let val_type = infer_expr_type_simple(value, local_symbols);
            local_symbols.insert(name.clone(), VariableInfo { var_type: val_type, storage: Storage::Local(0) });
        }
        Stmt::IndexAssign { .. } => {}
        Stmt::If { then_branch, .. } => {
            for s in then_branch {
                pre_populate_locals(s, local_symbols);
            }
        }
        Stmt::While { body, .. } => {
            for s in body {
                pre_populate_locals(s, local_symbols);
            }
        }
        Stmt::TryCatch { try_block, catch_block } => {
            for s in try_block {
                pre_populate_locals(s, local_symbols);
            }
            for s in catch_block {
                pre_populate_locals(s, local_symbols);
            }
        }
        _ => {}
    }
}

fn infer_expr_type_simple(expr: &Expr, local_symbols: &HashMap<String, VariableInfo>) -> Type {
    match expr {
        Expr::Int(_) => Type::Int,
        Expr::Str(_) => Type::Str,
        Expr::Var(name) => local_symbols.get(name).map(|info| info.var_type.clone()).unwrap_or(Type::Int),
        Expr::ListLiteral(elements) => {
            if elements.is_empty() {
                Type::List(Box::new(Type::Int))
            } else {
                Type::List(Box::new(infer_expr_type_simple(&elements[0], local_symbols)))
            }
        }
        Expr::DictLiteral(entries) => {
            let mut map = HashMap::new();
            for entry in entries {
                if let Expr::Str(k) = &entry.0 {
                    let val_t = infer_expr_type_simple(&entry.1, local_symbols);
                    map.insert(k.clone(), val_t);
                }
            }
            Type::Dict(map)
        }
        Expr::ListIndex { list, index } => {
            let l_type = infer_expr_type_simple(list, local_symbols);
            match l_type {
                Type::List(t) => *t,
                Type::Dict(map) => {
                    if let Expr::Str(k) = &**index {
                        map.get(k).cloned().unwrap_or(Type::Int)
                    } else {
                        Type::Int
                    }
                }
                _ => Type::Int,
            }
        }
        Expr::ReadFile(_) => Type::Str,
        _ => Type::Int,
    }
}

fn infer_return_type(body: &[Stmt], scope_stack: &[HashMap<String, VariableInfo>]) -> Type {
    for stmt in body {
        if let Some(t) = get_stmt_return_type(stmt, scope_stack) {
            return t;
        }
    }
    Type::Int
}

fn get_stmt_return_type(stmt: &Stmt, scope_stack: &[HashMap<String, VariableInfo>]) -> Option<Type> {
    match stmt {
        Stmt::Return(Some(expr)) => Some(infer_expr_type_simple_stack(expr, scope_stack)),
        Stmt::If { then_branch, .. } => {
            for s in then_branch {
                if let Some(t) = get_stmt_return_type(s, scope_stack) {
                    return Some(t);
                }
            }
            None
        }
        Stmt::While { body, .. } => {
            for s in body {
                if let Some(t) = get_stmt_return_type(s, scope_stack) {
                    return Some(t);
                }
            }
            None
        }
        Stmt::TryCatch { try_block, catch_block } => {
            for s in try_block {
                if let Some(t) = get_stmt_return_type(s, scope_stack) {
                    return Some(t);
                }
            }
            for s in catch_block {
                if let Some(t) = get_stmt_return_type(s, scope_stack) {
                    return Some(t);
                }
            }
            None
        }
        _ => None,
    }
}

fn infer_expr_type_simple_stack(expr: &Expr, scope_stack: &[HashMap<String, VariableInfo>]) -> Type {
    match expr {
        Expr::Int(_) => Type::Int,
        Expr::Str(_) => Type::Str,
        Expr::Var(name) => resolve_variable(name, scope_stack).map(|info| info.var_type).unwrap_or(Type::Int),
        Expr::ListLiteral(elements) => {
            if elements.is_empty() {
                Type::List(Box::new(Type::Int))
            } else {
                Type::List(Box::new(infer_expr_type_simple_stack(&elements[0], scope_stack)))
            }
        }
        Expr::DictLiteral(entries) => {
            let mut map = HashMap::new();
            for entry in entries {
                if let Expr::Str(k) = &entry.0 {
                    let val_t = infer_expr_type_simple_stack(&entry.1, scope_stack);
                    map.insert(k.clone(), val_t);
                }
            }
            Type::Dict(map)
        }
        Expr::ListIndex { list, index } => {
            let l_type = infer_expr_type_simple_stack(list, scope_stack);
            match l_type {
                Type::List(t) => *t,
                Type::Dict(map) => {
                    if let Expr::Str(k) = &**index {
                        map.get(k).cloned().unwrap_or(Type::Int)
                    } else {
                        Type::Int
                    }
                }
                _ => Type::Int,
            }
        }
        Expr::ReadFile(_) => Type::Str,
        _ => Type::Int,
    }
}

fn resolve_variable(name: &str, scope_stack: &[HashMap<String, VariableInfo>]) -> Option<VariableInfo> {
    for scope in scope_stack.iter().rev() {
        if let Some(info) = scope.get(name) {
            return Some(info.clone());
        }
    }
    None
}

fn set_variable(name: &str, var_info: VariableInfo, scope_stack: &mut [HashMap<String, VariableInfo>]) {
    for scope in scope_stack.iter_mut().rev() {
        if scope.contains_key(name) {
            scope.insert(name.to_string(), var_info.clone());
            return;
        }
    }
    if let Some(scope) = scope_stack.last_mut() {
        scope.insert(name.to_string(), var_info);
    }
}

fn type_check_stmt(
    stmt: &Stmt,
    scope_stack: &mut Vec<HashMap<String, VariableInfo>>,
    function_signatures: &HashMap<String, (Vec<Type>, Type)>,
    current_function: Option<&(String, Type)>,
) -> Result<(), TypeError> {
    match stmt {
        Stmt::Assign { name, value, name_line, name_col } => {
            let target_type = resolve_variable(name, scope_stack).map(|info| info.var_type);
            let val_type = type_check_expr(value, scope_stack, function_signatures, target_type.clone(), *name_line, *name_col)?;
            if let Some(existing_type) = target_type {
                if existing_type != val_type {
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
                let storage = if current_function.is_some() { Storage::Local(0) } else { Storage::Global };
                set_variable(name, VariableInfo { var_type: val_type, storage }, scope_stack);
            }
        }
        Stmt::IndexAssign { list, index, value, name_line, name_col } => {
            let list_type_opt = resolve_variable(list, scope_stack).map(|info| info.var_type.clone());
            match list_type_opt {
                Some(Type::List(t_type)) => {
                    type_check_expr(index, scope_stack, function_signatures, Some(Type::Int), *name_line, *name_col)?;
                    type_check_expr(value, scope_stack, function_signatures, Some(*t_type), *name_line, *name_col)?;
                }
                Some(Type::Dict(mut map)) => {
                    type_check_expr(index, scope_stack, function_signatures, Some(Type::Str), *name_line, *name_col)?;
                    let val_type = type_check_expr(value, scope_stack, function_signatures, None, *name_line, *name_col)?;
                    // Update key-value mapping in compile-time dictionary type representation
                    if let Expr::Str(k) = index {
                        map.insert(k.clone(), val_type.clone());
                        let storage = resolve_variable(list, scope_stack).unwrap().storage;
                        set_variable(list, VariableInfo { var_type: Type::Dict(map), storage }, scope_stack);
                    }
                }
                _ => {
                    return Err(TypeError {
                        message: format!("Variable '{}' is not indexable. Cannot assign an index.", list),
                        line: *name_line,
                        column: *name_col,
                        suggestion: "Ensure the variable is a List or a Dictionary before indexing it.".to_string(),
                    });
                }
            }
        }
        Stmt::Print(expr) => {
            type_check_expr(expr, scope_stack, function_signatures, None, 1, 1)?;
        }
        Stmt::If { cond, then_branch } => {
            let cond_type = type_check_expr(cond, scope_stack, function_signatures, Some(Type::Int), 1, 1)?;
            if cond_type != Type::Int {
                return Err(TypeError {
                    message: "The condition of an 'if' / 'wenn' statement must evaluate to a number (0 for false, non-zero for true).".to_string(),
                    line: 1,
                    column: 1,
                    suggestion: "Ensure the condition is a comparison or an integer value.".to_string(),
                });
            }
            for s in then_branch {
                type_check_stmt(s, scope_stack, function_signatures, current_function)?;
            }
        }
        Stmt::While { cond, body } => {
            let cond_type = type_check_expr(cond, scope_stack, function_signatures, Some(Type::Int), 1, 1)?;
            if cond_type != Type::Int {
                return Err(TypeError {
                    message: "The condition of a 'while' / 'solange' statement must evaluate to a number (0 for false, non-zero for true).".to_string(),
                    line: 1,
                    column: 1,
                    suggestion: "Ensure the condition is a comparison or an integer value.".to_string(),
                });
            }
            for s in body {
                type_check_stmt(s, scope_stack, function_signatures, current_function)?;
            }
        }
        Stmt::FnDef { name, params, body } => {
            let (_, ret_type) = function_signatures.get(name).unwrap();
            let mut local_symbols = HashMap::new();
            for p in params {
                local_symbols.insert(p.clone(), VariableInfo { var_type: Type::Int, storage: Storage::Local(0) });
            }
            for s in body {
                pre_populate_locals(s, &mut local_symbols);
            }
            
            scope_stack.push(local_symbols);
            for s in body {
                type_check_stmt(s, scope_stack, function_signatures, Some(&(name.clone(), ret_type.clone())))?;
            }
            scope_stack.pop();
        }
        Stmt::Return(expr_opt) => {
            if let Some((func_name, ret_type)) = current_function {
                let val_type = if let Some(expr) = expr_opt {
                    type_check_expr(expr, scope_stack, function_signatures, Some(ret_type.clone()), 1, 1)?
                } else {
                    Type::Int
                };
                if &val_type != ret_type {
                    return Err(TypeError {
                        message: format!(
                            "Function '{}' is declared to return {:?}, but returns {:?}",
                            func_name, ret_type, val_type
                        ),
                        line: 1,
                        column: 1,
                        suggestion: "Make sure all return statements in the function return values of the same type.".to_string(),
                    });
                }
            } else {
                return Err(TypeError {
                    message: "Return statement found outside function body.".to_string(),
                    line: 1,
                    column: 1,
                    suggestion: "Only use 'return' / 'rueckgabe' / 'zurueck' inside function declarations.".to_string(),
                });
            }
        }
        Stmt::Append { list, item } => {
            let list_type = type_check_expr(list, scope_stack, function_signatures, None, 1, 1)?;
            let item_expected = match &list_type {
                Type::List(t) => Some((**t).clone()),
                _ => {
                    return Err(TypeError {
                        message: "The first argument of 'append' / 'anfuegen' must be a List.".to_string(),
                        line: 1,
                        column: 1,
                        suggestion: "Ensure you are passing a list variable, e.g., 'append(my_list, item)'.".to_string(),
                    });
                }
            };
            let item_type = type_check_expr(item, scope_stack, function_signatures, item_expected.clone(), 1, 1)?;
            if Some(&item_type) != item_expected.as_ref() {
                return Err(TypeError {
                    message: format!("Type mismatch in append. Cannot append a {:?} to a List of {:?}.", item_type, item_expected.unwrap()),
                    line: 1,
                    column: 1,
                    suggestion: "Make sure the appended item type matches the list elements type.".to_string(),
                });
            }
        }
        Stmt::WriteFile { filename, content } => {
            type_check_expr(filename, scope_stack, function_signatures, Some(Type::Str), 1, 1)?;
            type_check_expr(content, scope_stack, function_signatures, Some(Type::Str), 1, 1)?;
        }
        Stmt::TryCatch { try_block, catch_block } => {
            for s in try_block {
                type_check_stmt(s, scope_stack, function_signatures, current_function)?;
            }
            
            for s in catch_block {
                type_check_stmt(s, scope_stack, function_signatures, current_function)?;
            }
        }
        Stmt::Expr(expr) => {
            type_check_expr(expr, scope_stack, function_signatures, None, 1, 1)?;
        }
    }
    Ok(())
}

fn type_check_expr(
    expr: &Expr,
    scope_stack: &mut Vec<HashMap<String, VariableInfo>>,
    function_signatures: &HashMap<String, (Vec<Type>, Type)>,
    expected_type: Option<Type>,
    line: usize,
    column: usize,
) -> Result<Type, TypeError> {
    match expr {
        Expr::Int(_) => Ok(Type::Int),
        Expr::Str(_) => Ok(Type::Str),
        Expr::Var(name) => {
            if let Some(info) = resolve_variable(name, scope_stack) {
                Ok(info.var_type)
            } else {
                Err(TypeError {
                    message: format!("Variable '{}' is used before being set.", name),
                    line,
                    column,
                    suggestion: format!("Initialize the variable first using 'set {} to [value]' (or 'setze {} auf [value]').", name, name),
                })
            }
        }
        Expr::Input { .. } => {
            Ok(expected_type.unwrap_or(Type::Str))
        }
        Expr::Len(sub) => {
            let sub_type = type_check_expr(sub, scope_stack, function_signatures, Some(Type::Str), line, column)?;
            if sub_type != Type::Str {
                return Err(TypeError {
                    message: "The 'len' / 'laenge' function requires a string argument.".to_string(),
                    line,
                    column,
                    suggestion: "Pass a string variable or a string literal to the function.".to_string(),
                });
            }
            Ok(Type::Int)
        }
        Expr::Sleep(sub) => {
            let sub_type = type_check_expr(sub, scope_stack, function_signatures, Some(Type::Int), line, column)?;
            if sub_type != Type::Int {
                return Err(TypeError {
                    message: "The 'sleep' / 'warte' function requires a number argument (milliseconds).".to_string(),
                    line,
                    column,
                    suggestion: "Pass an integer variable or an integer literal representing milliseconds.".to_string(),
                });
            }
            Ok(Type::Int)
        }
        Expr::Random => {
            Ok(Type::Int)
        }
        Expr::Alert { title, message } => {
            let t_type = type_check_expr(title, scope_stack, function_signatures, Some(Type::Str), line, column)?;
            let m_type = type_check_expr(message, scope_stack, function_signatures, Some(Type::Str), line, column)?;
            if t_type != Type::Str || m_type != Type::Str {
                return Err(TypeError {
                    message: "The 'alert' / 'info' function requires two string arguments (title and message).".to_string(),
                    line,
                    column,
                    suggestion: "Pass string literals or variables containing strings to alert/info.".to_string(),
                });
            }
            Ok(Type::Int)
        }
        Expr::Window { title, width, height } => {
            let t_type = type_check_expr(title, scope_stack, function_signatures, Some(Type::Str), line, column)?;
            let w_type = type_check_expr(width, scope_stack, function_signatures, Some(Type::Int), line, column)?;
            let h_type = type_check_expr(height, scope_stack, function_signatures, Some(Type::Int), line, column)?;
            if t_type != Type::Str || w_type != Type::Int || h_type != Type::Int {
                return Err(TypeError {
                    message: "The 'window' / 'fenster' function requires a string (title) and two numbers (width and height).".to_string(),
                    line,
                    column,
                    suggestion: "Example: window(\"My Window\", 800, 600)".to_string(),
                });
            }
            Ok(Type::Int)
        }
        Expr::Call { name, args } => {
            if let Some((param_types, ret_type)) = function_signatures.get(name) {
                if args.len() != param_types.len() {
                    return Err(TypeError {
                        message: format!("Function '{}' expects {} arguments, but got {}.", name, param_types.len(), args.len()),
                        line,
                        column,
                        suggestion: format!("Check the parameters of function '{}' and pass the correct number of values.", name),
                    });
                }
                for (i, arg) in args.iter().enumerate() {
                    let arg_type = type_check_expr(arg, scope_stack, function_signatures, Some(param_types[i].clone()), line, column)?;
                    if arg_type != param_types[i] {
                        return Err(TypeError {
                            message: format!("Argument {} in call to '{}' must be of type {:?}.", i + 1, name, param_types[i]),
                            line,
                            column,
                            suggestion: "Make sure the argument types match the function definition.".to_string(),
                        });
                    }
                }
                Ok(ret_type.clone())
            } else {
                Err(TypeError {
                    message: format!("Call to undefined function '{}'.", name),
                    line,
                    column,
                    suggestion: format!("Define the function using 'fn {}()' or check the spelling.", name),
                })
            }
        }
        Expr::ListLiteral(elements) => {
            if elements.is_empty() {
                Ok(Type::List(Box::new(Type::Int)))
            } else {
                let first_type = type_check_expr(&elements[0], scope_stack, function_signatures, None, line, column)?;
                for el in &elements[1..] {
                    let el_type = type_check_expr(el, scope_stack, function_signatures, Some(first_type.clone()), line, column)?;
                    if el_type != first_type {
                        return Err(TypeError {
                            message: format!("Type mismatch in list literal. Expected {:?}, but found {:?}.", first_type, el_type),
                            line,
                            column,
                            suggestion: "Ensure all elements in a list literal have the same type.".to_string(),
                        });
                    }
                }
                Ok(Type::List(Box::new(first_type)))
            }
        }
        Expr::DictLiteral(entries) => {
            let mut map = HashMap::new();
            for entry in entries {
                let k_type = type_check_expr(&entry.0, scope_stack, function_signatures, Some(Type::Str), line, column)?;
                if k_type != Type::Str {
                    return Err(TypeError {
                        message: "Dictionary keys must be strings.".to_string(),
                        line,
                        column,
                        suggestion: "Ensure all keys in the dictionary literal evaluate to string values.".to_string(),
                    });
                }
                
                if let Expr::Str(k_name) = &entry.0 {
                    let val_type = type_check_expr(&entry.1, scope_stack, function_signatures, None, line, column)?;
                    map.insert(k_name.clone(), val_type);
                }
            }
            Ok(Type::Dict(map))
        }
        Expr::ListIndex { list, index } => {
            let list_type = type_check_expr(list, scope_stack, function_signatures, None, line, column)?;
            match list_type {
                Type::List(t) => {
                    let idx_type = type_check_expr(index, scope_stack, function_signatures, Some(Type::Int), line, column)?;
                    if idx_type != Type::Int {
                        return Err(TypeError {
                            message: "Array index must be a number.".to_string(),
                            line,
                            column,
                            suggestion: "Use an integer index for list lookups.".to_string(),
                        });
                    }
                    Ok(*t)
                }
                Type::Dict(map) => {
                    let idx_type = type_check_expr(index, scope_stack, function_signatures, Some(Type::Str), line, column)?;
                    if idx_type != Type::Str {
                        return Err(TypeError {
                            message: "Dictionary key must be a string.".to_string(),
                            line,
                            column,
                            suggestion: "Use a string key for dictionary lookups.".to_string(),
                        });
                    }
                    if let Expr::Str(k_name) = &**index {
                        Ok(map.get(k_name).cloned().unwrap_or(Type::Int))
                    } else {
                        Ok(Type::Int)
                    }
                }
                _ => Err(TypeError {
                    message: "Cannot index a non-list/non-dict variable.".to_string(),
                    line,
                    column,
                    suggestion: "Verify that the variable being indexed is a list or a dictionary.".to_string(),
                }),
            }
        }
        Expr::ReadFile(sub) => {
            let sub_type = type_check_expr(sub, scope_stack, function_signatures, Some(Type::Str), line, column)?;
            if sub_type != Type::Str {
                return Err(TypeError {
                    message: "The 'read_file' function requires a string filename.".to_string(),
                    line,
                    column,
                    suggestion: "Ensure the parameter evaluates to a string.".to_string(),
                });
            }
            Ok(Type::Str)
        }
        Expr::Binary { op, left, right } => {
            let left_expected = match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div => Some(Type::Int),
                _ => None,
            };
            let right_expected = match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div => Some(Type::Int),
                _ => None,
            };
            let left_type = type_check_expr(left, scope_stack, function_signatures, left_expected, line, column)?;
            let right_type = type_check_expr(right, scope_stack, function_signatures, right_expected, line, column)?;
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
        Stmt::IndexAssign { index, value, .. } => {
            collect_expr_strings(index, literals);
            collect_expr_strings(value, literals);
        }
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
        Stmt::FnDef { body, .. } => {
            for s in body {
                collect_stmt_strings(s, literals);
            }
        }
        Stmt::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                collect_expr_strings(expr, literals);
            }
        }
        Stmt::Append { list, item } => {
            collect_expr_strings(list, literals);
            collect_expr_strings(item, literals);
        }
        Stmt::WriteFile { filename, content } => {
            collect_expr_strings(filename, literals);
            collect_expr_strings(content, literals);
        }
        Stmt::TryCatch { try_block, catch_block } => {
            for s in try_block {
                collect_stmt_strings(s, literals);
            }
            for s in catch_block {
                collect_stmt_strings(s, literals);
            }
        }
        Stmt::Expr(expr) => collect_expr_strings(expr, literals),
    }
}

fn collect_expr_strings(expr: &Expr, literals: &mut Vec<String>) {
    match expr {
        Expr::Str(s) => {
            if !literals.contains(s) {
                literals.push(s.clone());
            }
        }
        Expr::Len(sub) => collect_expr_strings(sub, literals),
        Expr::Sleep(sub) => collect_expr_strings(sub, literals),
        Expr::Alert { title, message } => {
            collect_expr_strings(title, literals);
            collect_expr_strings(message, literals);
        }
        Expr::Window { title, width, height } => {
            collect_expr_strings(title, literals);
            collect_expr_strings(width, literals);
            collect_expr_strings(height, literals);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_expr_strings(arg, literals);
            }
        }
        Expr::ListLiteral(elements) => {
            for el in elements {
                collect_expr_strings(el, literals);
            }
        }
        Expr::DictLiteral(entries) => {
            for entry in entries {
                collect_expr_strings(&entry.0, literals);
                collect_expr_strings(&entry.1, literals);
            }
        }
        Expr::ListIndex { list, index } => {
            collect_expr_strings(list, literals);
            collect_expr_strings(index, literals);
        }
        Expr::ReadFile(sub) => collect_expr_strings(sub, literals),
        Expr::Binary { left, right, .. } => {
            collect_expr_strings(left, literals);
            collect_expr_strings(right, literals);
        }
        _ => {}
    }
}

fn collect_input_ids(program: &Program, symbol_table: &HashMap<String, VariableInfo>) -> Vec<usize> {
    let mut ids = Vec::new();
    for stmt in &program.statements {
        collect_stmt_inputs(stmt, symbol_table, &mut ids);
    }
    ids
}

fn collect_stmt_inputs(stmt: &Stmt, symbol_table: &HashMap<String, VariableInfo>, ids: &mut Vec<usize>) {
    match stmt {
        Stmt::Assign { name, value, .. } => {
            let is_int_input = if let Expr::Input { .. } = value {
                symbol_table.get(name).map(|info| info.var_type.clone()) == Some(Type::Int)
            } else {
                false
            };
            if !is_int_input {
                collect_expr_inputs(value, ids);
            }
        }
        Stmt::IndexAssign { value, .. } => {
            collect_expr_inputs(value, ids);
        }
        Stmt::Print(expr) => collect_expr_inputs(expr, ids),
        Stmt::If { cond, then_branch } => {
            collect_expr_inputs(cond, ids);
            for s in then_branch {
                collect_stmt_inputs(s, symbol_table, ids);
            }
        }
        Stmt::While { cond, body } => {
            collect_expr_inputs(cond, ids);
            for s in body {
                collect_stmt_inputs(s, symbol_table, ids);
            }
        }
        Stmt::FnDef { body, .. } => {
            for s in body {
                collect_stmt_inputs(s, symbol_table, ids);
            }
        }
        Stmt::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                collect_expr_inputs(expr, ids);
            }
        }
        Stmt::Append { list, item } => {
            collect_expr_inputs(list, ids);
            collect_expr_inputs(item, ids);
        }
        Stmt::WriteFile { filename, content } => {
            collect_expr_inputs(filename, ids);
            collect_expr_inputs(content, ids);
        }
        Stmt::TryCatch { try_block, catch_block } => {
            for s in try_block {
                collect_stmt_inputs(s, symbol_table, ids);
            }
            for s in catch_block {
                collect_stmt_inputs(s, symbol_table, ids);
            }
        }
        Stmt::Expr(expr) => collect_expr_inputs(expr, ids),
    }
}

fn collect_expr_inputs(expr: &Expr, ids: &mut Vec<usize>) {
    match expr {
        Expr::Input { id } => {
            if !ids.contains(id) {
                ids.push(*id);
            }
        }
        Expr::Len(sub) => collect_expr_inputs(sub, ids),
        Expr::Sleep(sub) => collect_expr_inputs(sub, ids),
        Expr::Alert { title, message } => {
            collect_expr_inputs(title, ids);
            collect_expr_inputs(message, ids);
        }
        Expr::Window { title, width, height } => {
            collect_expr_inputs(title, ids);
            collect_expr_inputs(width, ids);
            collect_expr_inputs(height, ids);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_expr_inputs(arg, ids);
            }
        }
        Expr::ListLiteral(elements) => {
            for el in elements {
                collect_expr_inputs(el, ids);
            }
        }
        Expr::DictLiteral(entries) => {
            for entry in entries {
                collect_expr_inputs(&entry.0, ids);
                collect_expr_inputs(&entry.1, ids);
            }
        }
        Expr::ListIndex { list, index } => {
            collect_expr_inputs(list, ids);
            collect_expr_inputs(index, ids);
        }
        Expr::ReadFile(sub) => collect_expr_inputs(sub, ids),
        Expr::Binary { left, right, .. } => {
            collect_expr_inputs(left, ids);
            collect_expr_inputs(right, ids);
        }
        _ => {}
    }
}

fn has_gui_calls(program: &Program) -> bool {
    for s in &program.statements {
        if stmt_has_gui(s) {
            return true;
        }
    }
    false
}

fn stmt_has_gui(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Assign { value, .. } => expr_has_gui(value),
        Stmt::IndexAssign { value, .. } => expr_has_gui(value),
        Stmt::Print(expr) => expr_has_gui(expr),
        Stmt::If { cond, then_branch } => {
            if expr_has_gui(cond) {
                return true;
            }
            for s in then_branch {
                if stmt_has_gui(s) {
                    return true;
                }
            }
            false
        }
        Stmt::While { cond, body } => {
            if expr_has_gui(cond) {
                return true;
            }
            for s in body {
                if stmt_has_gui(s) {
                    return true;
                }
            }
            false
        }
        Stmt::FnDef { body, .. } => {
            for s in body {
                if stmt_has_gui(s) {
                    return true;
                }
            }
            false
        }
        Stmt::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                expr_has_gui(expr)
            } else {
                false
            }
        }
        Stmt::Append { list, item } => {
            expr_has_gui(list) || expr_has_gui(item)
        }
        Stmt::WriteFile { filename, content } => {
            expr_has_gui(filename) || expr_has_gui(content)
        }
        Stmt::TryCatch { try_block, catch_block } => {
            for s in try_block {
                if stmt_has_gui(s) {
                    return true;
                }
            }
            for s in catch_block {
                if stmt_has_gui(s) {
                    return true;
                }
            }
            false
        }
        Stmt::Expr(expr) => expr_has_gui(expr),
    }
}

fn expr_has_gui(expr: &Expr) -> bool {
    match expr {
        Expr::Alert { .. } | Expr::Window { .. } => true,
        Expr::Len(sub) => expr_has_gui(sub),
        Expr::Sleep(sub) => expr_has_gui(sub),
        Expr::Call { args, .. } => {
            for arg in args {
                if expr_has_gui(arg) {
                    return true;
                }
            }
            false
        }
        Expr::ListLiteral(elements) => {
            for el in elements {
                if expr_has_gui(el) {
                    return true;
                }
            }
            false
        }
        Expr::DictLiteral(entries) => {
            for entry in entries {
                if expr_has_gui(&entry.0) || expr_has_gui(&entry.1) {
                    return true;
                }
            }
            false
        }
        Expr::ListIndex { list, index } => {
            expr_has_gui(list) || expr_has_gui(index)
        }
        Expr::ReadFile(sub) => expr_has_gui(sub),
        Expr::Binary { left, right, .. } => expr_has_gui(left) || expr_has_gui(right),
        _ => false,
    }
}

fn escape_fasm_string(s: &str) -> String {
    s.replace("'", "''")
}

fn get_expr_type(
    expr: &Expr,
    symbol_table: &HashMap<String, VariableInfo>,
    function_signatures: &HashMap<String, (Vec<Type>, Type)>,
    expected_type: Option<Type>,
) -> Type {
    match expr {
        Expr::Int(_) => Type::Int,
        Expr::Str(_) => Type::Str,
        Expr::Var(name) => symbol_table.get(name).map(|info| info.var_type.clone()).unwrap_or(Type::Int),
        Expr::Input { .. } => expected_type.unwrap_or(Type::Str),
        Expr::Len(_) => Type::Int,
        Expr::Sleep(_) => Type::Int,
        Expr::Random => Type::Int,
        Expr::Alert { .. } => Type::Int,
        Expr::Window { .. } => Type::Int,
        Expr::Call { name, .. } => {
            if let Some((_, ret_type)) = function_signatures.get(name) {
                ret_type.clone()
            } else {
                Type::Int
            }
        }
        Expr::ListLiteral(elements) => {
            if elements.is_empty() {
                Type::List(Box::new(Type::Int))
            } else {
                Type::List(Box::new(get_expr_type(&elements[0], symbol_table, function_signatures, expected_type)))
            }
        }
        Expr::DictLiteral(entries) => {
            let mut map = HashMap::new();
            for entry in entries {
                if let Expr::Str(k_name) = &entry.0 {
                    let val_type = get_expr_type(&entry.1, symbol_table, function_signatures, expected_type.clone());
                    map.insert(k_name.clone(), val_type);
                }
            }
            Type::Dict(map)
        }
        Expr::ListIndex { list, index } => {
            let l_type = get_expr_type(list, symbol_table, function_signatures, expected_type);
            match l_type {
                Type::List(t) => *t,
                Type::Dict(map) => {
                    if let Expr::Str(k_name) = &**index {
                        map.get(k_name).cloned().unwrap_or(Type::Int)
                    } else {
                        Type::Int
                    }
                }
                _ => Type::Int,
            }
        }
        Expr::ReadFile(_) => Type::Str,
        Expr::Binary { op, .. } => {
            match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div => Type::Int,
                _ => Type::Int,
            }
        }
    }
}

fn collect_local_vars(body: &[Stmt], params: &[String]) -> Vec<String> {
    let mut locals = Vec::new();
    for stmt in body {
        collect_stmt_locals(stmt, params, &mut locals);
    }
    locals
}

fn collect_stmt_locals(stmt: &Stmt, params: &[String], locals: &mut Vec<String>) {
    match stmt {
        Stmt::Assign { name, .. } => {
            if !params.contains(name) && !locals.contains(name) {
                locals.push(name.clone());
            }
        }
        Stmt::IndexAssign { .. } => {}
        Stmt::If { then_branch, .. } => {
            for s in then_branch {
                collect_stmt_locals(s, params, locals);
            }
        }
        Stmt::While { body, .. } => {
            for s in body {
                collect_stmt_locals(s, params, locals);
            }
        }
        Stmt::TryCatch { try_block, catch_block } => {
            for s in try_block {
                collect_stmt_locals(s, params, locals);
            }
            for s in catch_block {
                collect_stmt_locals(s, params, locals);
            }
        }
        _ => {}
    }
}

pub fn generate_assembly(program: &Program, symbol_table: &HashMap<String, VariableInfo>) -> String {
    let mut asm = String::new();
    let has_gui = has_gui_calls(program);
    let stack_res = if has_gui { 96 } else { 32 };
    
    // Construct function signatures
    let mut function_signatures = HashMap::new();
    for stmt in &program.statements {
        if let Stmt::FnDef { name, params, body } = stmt {
            let mut local_symbols = HashMap::new();
            for p in params {
                local_symbols.insert(p.clone(), VariableInfo { var_type: Type::Int, storage: Storage::Local(0) });
            }
            for s in body {
                pre_populate_locals(s, &mut local_symbols);
            }
            let body_symbols_stack = vec![HashMap::new(), local_symbols];
            let ret_type = infer_return_type(body, &body_symbols_stack);
            let param_types = vec![Type::Int; params.len()];
            function_signatures.insert(name.clone(), (param_types, ret_type));
        }
    }
    
    // Header
    asm.push_str("format PE64 console\n");
    asm.push_str("entry start\n\n");
    
    // String literals mapping
    let string_literals = collect_string_literals(program);
    let mut string_map = HashMap::new();
    for (idx, lit) in string_literals.iter().enumerate() {
        string_map.insert(lit.clone(), idx);
    }
    
    // Collect input buffer IDs
    let input_ids = collect_input_ids(program, symbol_table);
    
    // Section .data
    asm.push_str("section '.data' data readable writeable\n");
    asm.push_str("  fmt_int db '%lld', 13, 10, 0\n");
    asm.push_str("  fmt_str db '%s', 13, 10, 0\n");
    asm.push_str("  fmt_int_in db '%lld', 0\n");
    asm.push_str("  fmt_str_in db '%s', 0\n");
    asm.push_str("  bounds_err_msg db 'Error: Array index out of bounds.', 13, 10, 0\n");
    asm.push_str("  dict_err_msg db 'Error: Dictionary key not found.', 13, 10, 0\n");
    asm.push_str("  file_err_msg db 'Error: Could not open file.', 13, 10, 0\n");
    asm.push_str("  file_mode_w db 'w', 0\n");
    asm.push_str("  file_mode_rb db 'rb', 0\n");
    asm.push_str("  bro_error_code dq 0\n");
    asm.push_str("  bro_exception_handler dq 0\n");
    asm.push_str("  exception_depth dq 0\n");
    asm.push_str("  saved_rsps dq 32 dup 0\n");
    asm.push_str("  saved_handlers dq 32 dup 0\n");
    
    for (lit, idx) in &string_map {
        let escaped = escape_fasm_string(lit);
        asm.push_str(&format!("  str_lit_{} db '{}', 0\n", idx, escaped));
    }
    
    for id in &input_ids {
        asm.push_str(&format!("  input_buf_{} db 256 dup 0\n", id));
    }
    
    // Global variables in symbol_table
    for (var_name, var_info) in symbol_table {
        if let Storage::Global = var_info.storage {
            asm.push_str(&format!("  var_{} dq 0\n", var_name));
        }
    }
    
    if has_gui {
        asm.push_str("  window_class_name db 'BroLangWndClass', 0\n");
        asm.push_str("  msg_struct rb 48\n");
        asm.push_str("  wnd_class:\n");
        asm.push_str("    wc_style         dd 0\n");
        asm.push_str("                     dd 0 ; alignment\n");
        asm.push_str("    wc_lpfnWndProc   dq rva window_proc\n");
        asm.push_str("    wc_cbClsExtra    dd 0\n");
        asm.push_str("    wc_cbWndExtra    dd 0\n");
        asm.push_str("    wc_hInstance     dq 0\n");
        asm.push_str("    wc_hIcon         dq 0\n");
        asm.push_str("    wc_hCursor       dq 0\n");
        asm.push_str("    wc_hbrBackground dq 6\n"); // COLOR_WINDOW+1
        asm.push_str("    wc_lpszMenuName  dq 0\n");
        asm.push_str("    wc_lpszClassName dq window_class_name\n");
    }
    asm.push_str("\n");
    
    // Section .text
    asm.push_str("section '.text' code readable executable\n");
    asm.push_str("start:\n");
    asm.push_str("  push rbp\n");
    asm.push_str("  mov rbp, rsp\n");
    asm.push_str("  and rsp, -16\n"); // Align stack to 16-byte boundary
    asm.push_str(&format!("  sub rsp, {}\n\n", stack_res));
    
    if has_gui {
        asm.push_str("  ; Register Win32 Class\n");
        asm.push_str("  mov rcx, wnd_class\n");
        asm.push_str("  call [RegisterClassA]\n\n");
    }
    
    let mut label_counter = 0;
    let mut functions_asm = String::new();
    
    for stmt in &program.statements {
        if let Stmt::FnDef { name, params, body } = stmt {
            let locals = collect_local_vars(body, params);
            let local_bytes = 8 * locals.len();
            let total_needed = 32 + local_bytes;
            let reservation_size = if total_needed % 16 == 0 {
                total_needed
            } else {
                total_needed + 8
            };
            
            let mut local_offsets = HashMap::new();
            for (i, p) in params.iter().enumerate() {
                local_offsets.insert(p.clone(), 16 + 8 * (i as i32));
            }
            for (i, l) in locals.iter().enumerate() {
                local_offsets.insert(l.clone(), -8 - 8 * (i as i32));
            }
            
            let ctx = CodegenCtx {
                local_offsets,
                func_name: name.clone(),
            };
            
            functions_asm.push_str(&format!("fn_{}:\n", name));
            functions_asm.push_str("  push rbp\n");
            functions_asm.push_str("  mov rbp, rsp\n");
            functions_asm.push_str("  and rsp, -16\n"); // Align stack in function as well
            functions_asm.push_str(&format!("  sub rsp, {}\n", reservation_size));
            
            if params.len() >= 1 {
                functions_asm.push_str("  mov [rbp + 16], rcx\n");
            }
            if params.len() >= 2 {
                functions_asm.push_str("  mov [rbp + 24], rdx\n");
            }
            if params.len() >= 3 {
                functions_asm.push_str("  mov [rbp + 32], r8\n");
            }
            if params.len() >= 4 {
                functions_asm.push_str("  mov [rbp + 40], r9\n");
            }
            
            for s in body {
                codegen_stmt_with_ctx(s, &mut functions_asm, &string_map, symbol_table, &function_signatures, &mut label_counter, Some(&ctx));
            }
            
            functions_asm.push_str(&format!(".L_epilogue_{}:\n", name));
            functions_asm.push_str("  mov rsp, rbp\n");
            functions_asm.push_str("  pop rbp\n");
            functions_asm.push_str("  ret\n\n");
        } else {
            codegen_stmt_with_ctx(stmt, &mut asm, &string_map, symbol_table, &function_signatures, &mut label_counter, None);
        }
    }
    
    asm.push_str("  mov rcx, 0\n");
    asm.push_str("  call [ExitProcess]\n\n");
    
    // Bounds check error handler
    asm.push_str(".L_bounds_error:\n");
    asm.push_str("  mov rcx, [bro_exception_handler]\n");
    asm.push_str("  test rcx, rcx\n");
    asm.push_str("  jz .L_bounds_error_crash\n");
    asm.push_str("  mov qword [bro_error_code], 1\n");
    asm.push_str("  jmp rcx\n");
    asm.push_str(".L_bounds_error_crash:\n");
    asm.push_str("  mov rdx, bounds_err_msg\n");
    asm.push_str("  mov rcx, fmt_str\n");
    asm.push_str("  call [printf]\n");
    asm.push_str("  mov rcx, 1\n");
    asm.push_str("  call [ExitProcess]\n\n");
    
    // File I/O error handler
    asm.push_str(".L_file_error:\n");
    asm.push_str("  mov rcx, [bro_exception_handler]\n");
    asm.push_str("  test rcx, rcx\n");
    asm.push_str("  jz .L_file_error_crash\n");
    asm.push_str("  mov qword [bro_error_code], 1\n");
    asm.push_str("  jmp rcx\n");
    asm.push_str(".L_file_error_crash:\n");
    asm.push_str("  mov rdx, file_err_msg\n");
    asm.push_str("  mov rcx, fmt_str\n");
    asm.push_str("  call [printf]\n");
    asm.push_str("  mov rcx, 1\n");
    asm.push_str("  call [ExitProcess]\n\n");
    
    // Dictionary Subroutines
    asm.push_str(".L_fnv1a_hash:\n");
    asm.push_str("  mov rax, 0xcbf29ce484222325\n");
    asm.push_str("  mov rsi, rcx\n");
    asm.push_str(".L_hash_loop:\n");
    asm.push_str("  movzx rdi, byte [rsi]\n");
    asm.push_str("  test rdi, rdi\n");
    asm.push_str("  jz .L_hash_done\n");
    asm.push_str("  xor rax, rdi\n");
    asm.push_str("  mov rdx, 0x100000001b3\n");
    asm.push_str("  imul rax, rdx\n");
    asm.push_str("  inc rsi\n");
    asm.push_str("  jmp .L_hash_loop\n");
    asm.push_str(".L_hash_done:\n");
    asm.push_str("  ret\n\n");
    
    asm.push_str(".L_dict_insert:\n");
    asm.push_str("  push rbp\n");
    asm.push_str("  mov rbp, rsp\n");
    asm.push_str("  and rsp, -16\n");
    asm.push_str("  sub rsp, 48\n");
    asm.push_str("  mov [rbp - 8], rcx\n"); // dict_ptr
    asm.push_str("  mov [rbp - 16], rdx\n"); // key_str
    asm.push_str("  mov [rbp - 24], r8\n"); // value
    
    asm.push_str("  mov rcx, rdx\n");
    asm.push_str("  call .L_fnv1a_hash\n");
    asm.push_str("  and rax, 15\n");
    asm.push_str("  mov [rbp - 32], rax\n"); // bucket index
    
    asm.push_str("  mov rcx, [rbp - 8]\n");
    asm.push_str("  mov rax, [rbp - 32]\n");
    asm.push_str("  mov r12, [rcx + 8 * rax]\n"); // entry_ptr
    asm.push_str(".L_dict_insert_loop:\n");
    asm.push_str("  test r12, r12\n");
    asm.push_str("  jz .L_dict_insert_new\n");
    asm.push_str("  mov rdx, [rbp - 16]\n");
    asm.push_str("  mov rcx, [r12 + 0]\n");
    asm.push_str("  call [strcmp]\n");
    asm.push_str("  test rax, rax\n");
    asm.push_str("  jz .L_dict_insert_update\n");
    asm.push_str("  mov r12, [r12 + 16]\n");
    asm.push_str("  jmp .L_dict_insert_loop\n");
    
    asm.push_str(".L_dict_insert_update:\n");
    asm.push_str("  mov rax, [rbp - 24]\n");
    asm.push_str("  mov [r12 + 8], rax\n");
    asm.push_str("  jmp .L_dict_insert_done\n");
    
    asm.push_str(".L_dict_insert_new:\n");
    asm.push_str("  mov rcx, 24\n");
    asm.push_str("  call [malloc]\n");
    asm.push_str("  mov rdx, [rbp - 16]\n");
    asm.push_str("  mov [rax + 0], rdx\n");
    asm.push_str("  mov r8, [rbp - 24]\n");
    asm.push_str("  mov [rax + 8], r8\n");
    asm.push_str("  mov rcx, [rbp - 8]\n");
    asm.push_str("  mov rdx, [rbp - 32]\n");
    asm.push_str("  mov r9, [rcx + 8 * rdx]\n");
    asm.push_str("  mov [rax + 16], r9\n");
    asm.push_str("  mov [rcx + 8 * rdx], rax\n");
    
    asm.push_str(".L_dict_insert_done:\n");
    asm.push_str("  mov rsp, rbp\n");
    asm.push_str("  pop rbp\n");
    asm.push_str("  ret\n\n");
    
    asm.push_str(".L_dict_lookup:\n");
    asm.push_str("  push rbp\n");
    asm.push_str("  mov rbp, rsp\n");
    asm.push_str("  and rsp, -16\n");
    asm.push_str("  sub rsp, 48\n");
    asm.push_str("  mov [rbp - 8], rcx\n");
    asm.push_str("  mov [rbp - 16], rdx\n");
    
    asm.push_str("  test rcx, rcx\n");
    asm.push_str("  jz .L_dict_lookup_error\n");
    
    asm.push_str("  mov rcx, rdx\n");
    asm.push_str("  call .L_fnv1a_hash\n");
    asm.push_str("  and rax, 15\n");
    
    asm.push_str("  mov rcx, [rbp - 8]\n");
    asm.push_str("  mov r12, [rcx + 8 * rax]\n");
    asm.push_str(".L_dict_lookup_loop:\n");
    asm.push_str("  test r12, r12\n");
    asm.push_str("  jz .L_dict_lookup_error\n");
    asm.push_str("  mov rdx, [rbp - 16]\n");
    asm.push_str("  mov rcx, [r12 + 0]\n");
    asm.push_str("  call [strcmp]\n");
    asm.push_str("  test rax, rax\n");
    asm.push_str("  jz .L_dict_lookup_found\n");
    asm.push_str("  mov r12, [r12 + 16]\n");
    asm.push_str("  jmp .L_dict_lookup_loop\n");
    
    asm.push_str(".L_dict_lookup_found:\n");
    asm.push_str("  mov rax, [r12 + 8]\n");
    asm.push_str("  jmp .L_dict_lookup_done\n");
    
    asm.push_str(".L_dict_lookup_error:\n");
    asm.push_str("  mov rcx, [bro_exception_handler]\n");
    asm.push_str("  test rcx, rcx\n");
    asm.push_str("  jz .L_dict_lookup_crash\n");
    asm.push_str("  mov qword [bro_error_code], 1\n");
    asm.push_str("  jmp rcx\n");
    asm.push_str(".L_dict_lookup_crash:\n");
    asm.push_str("  mov rdx, dict_err_msg\n");
    asm.push_str("  mov rcx, fmt_str\n");
    asm.push_str("  call [printf]\n");
    asm.push_str("  mov rcx, 1\n");
    asm.push_str("  call [ExitProcess]\n");
    
    asm.push_str(".L_dict_lookup_done:\n");
    asm.push_str("  mov rsp, rbp\n");
    asm.push_str("  pop rbp\n");
    asm.push_str("  ret\n\n");
    
    // Compile functions
    asm.push_str(&functions_asm);
    
    if has_gui {
        asm.push_str("window_proc:\n");
        asm.push_str("  cmp rdx, 2 ; WM_DESTROY\n");
        asm.push_str("  je .L_destroy_wnd\n");
        asm.push_str("  sub rsp, 40\n");
        asm.push_str("  call [DefWindowProcA]\n");
        asm.push_str("  add rsp, 40\n");
        asm.push_str("  ret\n");
        asm.push_str(".L_destroy_wnd:\n");
        asm.push_str("  mov rcx, 0\n");
        asm.push_str("  call [ExitProcess]\n\n");
    }
    
    // Import Directory
    asm.push_str("section '.idata' import data readable\n\n");
    asm.push_str("  dd rva kernel32_lookup, 0, 0, rva kernel32_name, rva kernel32_address\n");
    asm.push_str("  dd rva msvcrt_lookup, 0, 0, rva msvcrt_name, rva msvcrt_address\n");
    if has_gui {
        asm.push_str("  dd rva user32_lookup, 0, 0, rva user32_name, rva user32_address\n");
    }
    asm.push_str("  dd 0, 0, 0, 0, 0\n\n");
    
    asm.push_str("  kernel32_lookup:\n");
    asm.push_str("    dq rva kernel32_ExitProcess\n");
    asm.push_str("    dq rva kernel32_Sleep\n");
    asm.push_str("    dq 0\n\n");
    asm.push_str("  kernel32_address:\n");
    asm.push_str("    ExitProcess dq rva kernel32_ExitProcess\n");
    asm.push_str("    Sleep       dq rva kernel32_Sleep\n");
    asm.push_str("    dq 0\n\n");
    
    asm.push_str("  msvcrt_lookup:\n");
    asm.push_str("    dq rva msvcrt_printf\n");
    asm.push_str("    dq rva msvcrt_scanf\n");
    asm.push_str("    dq rva msvcrt_strlen\n");
    asm.push_str("    dq rva msvcrt_rand\n");
    asm.push_str("    dq rva msvcrt_malloc\n");
    asm.push_str("    dq rva msvcrt_realloc\n");
    asm.push_str("    dq rva msvcrt_strcmp\n");
    asm.push_str("    dq rva msvcrt_fopen\n");
    asm.push_str("    dq rva msvcrt_fread\n");
    asm.push_str("    dq rva msvcrt_fwrite\n");
    asm.push_str("    dq rva msvcrt_fclose\n");
    asm.push_str("    dq rva msvcrt_fseek\n");
    asm.push_str("    dq rva msvcrt_ftell\n");
    asm.push_str("    dq 0\n\n");
    asm.push_str("  msvcrt_address:\n");
    asm.push_str("    printf      dq rva msvcrt_printf\n");
    asm.push_str("    scanf       dq rva msvcrt_scanf\n");
    asm.push_str("    strlen      dq rva msvcrt_strlen\n");
    asm.push_str("    rand        dq rva msvcrt_rand\n");
    asm.push_str("    malloc      dq rva msvcrt_malloc\n");
    asm.push_str("    realloc     dq rva msvcrt_realloc\n");
    asm.push_str("    strcmp      dq rva msvcrt_strcmp\n");
    asm.push_str("    fopen       dq rva msvcrt_fopen\n");
    asm.push_str("    fread       dq rva msvcrt_fread\n");
    asm.push_str("    fwrite      dq rva msvcrt_fwrite\n");
    asm.push_str("    fclose      dq rva msvcrt_fclose\n");
    asm.push_str("    fseek       dq rva msvcrt_fseek\n");
    asm.push_str("    ftell       dq rva msvcrt_ftell\n");
    asm.push_str("    dq 0\n\n");
    
    if has_gui {
        asm.push_str("  user32_lookup:\n");
        asm.push_str("    dq rva user32_MessageBoxA\n");
        asm.push_str("    dq rva user32_RegisterClassA\n");
        asm.push_str("    dq rva user32_CreateWindowExA\n");
        asm.push_str("    dq rva user32_DefWindowProcA\n");
        asm.push_str("    dq rva user32_GetMessageA\n");
        asm.push_str("    dq rva user32_TranslateMessage\n");
        asm.push_str("    dq rva user32_DispatchMessageA\n");
        asm.push_str("    dq 0\n\n");
        asm.push_str("  user32_address:\n");
        asm.push_str("    MessageBoxA      dq rva user32_MessageBoxA\n");
        asm.push_str("    RegisterClassA   dq rva user32_RegisterClassA\n");
        asm.push_str("    CreateWindowExA  dq rva user32_CreateWindowExA\n");
        asm.push_str("    DefWindowProcA   dq rva user32_DefWindowProcA\n");
        asm.push_str("    GetMessageA      dq rva user32_GetMessageA\n");
        asm.push_str("    TranslateMessage dq rva user32_TranslateMessage\n");
        asm.push_str("    DispatchMessageA dq rva user32_DispatchMessageA\n");
        asm.push_str("    dq 0\n\n");
    }
    
    asm.push_str("  kernel32_name db 'KERNEL32.DLL', 0\n");
    asm.push_str("  msvcrt_name   db 'MSVCRT.DLL', 0\n");
    if has_gui {
        asm.push_str("  user32_name   db 'USER32.DLL', 0\n");
    }
    asm.push_str("\n");
    
    asm.push_str("  kernel32_ExitProcess dw 0\n");
    asm.push_str("                       db 'ExitProcess', 0\n");
    asm.push_str("  kernel32_Sleep       dw 0\n");
    asm.push_str("                       db 'Sleep', 0\n\n");
    
    asm.push_str("  msvcrt_printf        dw 0\n");
    asm.push_str("                       db 'printf', 0\n");
    asm.push_str("  msvcrt_scanf         dw 0\n");
    asm.push_str("                       db 'scanf', 0\n");
    asm.push_str("  msvcrt_strlen        dw 0\n");
    asm.push_str("                       db 'strlen', 0\n");
    asm.push_str("  msvcrt_rand          dw 0\n");
    asm.push_str("                       db 'rand', 0\n");
    asm.push_str("  msvcrt_malloc        dw 0\n");
    asm.push_str("                       db 'malloc', 0\n");
    asm.push_str("  msvcrt_realloc       dw 0\n");
    asm.push_str("                       db 'realloc', 0\n");
    asm.push_str("  msvcrt_strcmp        dw 0\n");
    asm.push_str("                       db 'strcmp', 0\n");
    asm.push_str("  msvcrt_fopen         dw 0\n");
    asm.push_str("                       db 'fopen', 0\n");
    asm.push_str("  msvcrt_fread         dw 0\n");
    asm.push_str("                       db 'fread', 0\n");
    asm.push_str("  msvcrt_fwrite        dw 0\n");
    asm.push_str("                       db 'fwrite', 0\n");
    asm.push_str("  msvcrt_fclose        dw 0\n");
    asm.push_str("                       db 'fclose', 0\n");
    asm.push_str("  msvcrt_fseek         dw 0\n");
    asm.push_str("                       db 'fseek', 0\n");
    asm.push_str("  msvcrt_ftell         dw 0\n");
    asm.push_str("                       db 'ftell', 0\n\n");
    
    if has_gui {
        asm.push_str("  user32_MessageBoxA      dw 0\n");
        asm.push_str("                          db 'MessageBoxA', 0\n");
        asm.push_str("  user32_RegisterClassA   dw 0\n");
        asm.push_str("                          db 'RegisterClassA', 0\n");
        asm.push_str("  user32_CreateWindowExA  dw 0\n");
        asm.push_str("                          db 'CreateWindowExA', 0\n");
        asm.push_str("  user32_DefWindowProcA   dw 0\n");
        asm.push_str("                          db 'DefWindowProcA', 0\n");
        asm.push_str("  user32_GetMessageA      dw 0\n");
        asm.push_str("                          db 'GetMessageA', 0\n");
        asm.push_str("  user32_TranslateMessage dw 0\n");
        asm.push_str("                          db 'TranslateMessage', 0\n");
        asm.push_str("  user32_DispatchMessageA dw 0\n");
        asm.push_str("                          db 'DispatchMessageA', 0\n");
    }
    
    asm
}

fn codegen_stmt_with_ctx(
    stmt: &Stmt,
    asm: &mut String,
    string_map: &HashMap<String, usize>,
    symbol_table: &HashMap<String, VariableInfo>,
    function_signatures: &HashMap<String, (Vec<Type>, Type)>,
    label_counter: &mut usize,
    ctx: Option<&CodegenCtx>,
) {
    match stmt {
        Stmt::Assign { name, value, .. } => {
            if let Expr::Input { id } = value {
                let target_type = symbol_table.get(name).map(|info| info.var_type.clone()).unwrap_or(Type::Str);
                match target_type {
                    Type::Int => {
                        if let Some(c) = ctx {
                            if let Some(offset) = c.local_offsets.get(name) {
                                asm.push_str(&format!("  lea rdx, [rbp {}]\n", if *offset >= 0 { format!("+ {}", offset) } else { format!("- {}", -offset) }));
                            } else {
                                asm.push_str(&format!("  mov rdx, var_{}\n", name));
                            }
                        } else {
                            asm.push_str(&format!("  mov rdx, var_{}\n", name));
                        }
                        asm.push_str("  mov rcx, fmt_int_in\n");
                        asm.push_str("  call [scanf]\n");
                        if let Some(c) = ctx {
                            if let Some(offset) = c.local_offsets.get(name) {
                                asm.push_str(&format!("  mov rax, [rbp {}]\n", if *offset >= 0 { format!("+ {}", offset) } else { format!("- {}", -offset) }));
                            } else {
                                asm.push_str(&format!("  mov rax, [var_{}]\n", name));
                            }
                        } else {
                            asm.push_str(&format!("  mov rax, [var_{}]\n", name));
                        }
                    }
                    _ => {
                        asm.push_str(&format!("  mov rdx, input_buf_{}\n", id));
                        asm.push_str("  mov rcx, fmt_str_in\n");
                        asm.push_str("  call [scanf]\n");
                        asm.push_str(&format!("  mov rax, input_buf_{}\n", id));
                        if let Some(c) = ctx {
                            if let Some(offset) = c.local_offsets.get(name) {
                                asm.push_str(&format!("  mov [rbp {}], rax\n", if *offset >= 0 { format!("+ {}", offset) } else { format!("- {}", -offset) }));
                            } else {
                                asm.push_str(&format!("  mov [var_{}], rax\n", name));
                            }
                        } else {
                            asm.push_str(&format!("  mov [var_{}], rax\n", name));
                        }
                    }
                }
            } else {
                codegen_expr_with_ctx(value, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
                if let Some(c) = ctx {
                    if let Some(offset) = c.local_offsets.get(name) {
                        asm.push_str(&format!("  mov [rbp {}], rax\n", if *offset >= 0 { format!("+ {}", offset) } else { format!("- {}", -offset) }));
                    } else {
                        asm.push_str(&format!("  mov [var_{}], rax\n", name));
                    }
                } else {
                    asm.push_str(&format!("  mov [var_{}], rax\n", name));
                }
            }
        }
        Stmt::IndexAssign { list, index, value, .. } => {
            let list_type = symbol_table.get(list).map(|info| info.var_type.clone()).unwrap_or(Type::List(Box::new(Type::Int)));
            match list_type {
                Type::Dict(..) => {
                    codegen_expr_with_ctx(&Expr::Var(list.clone()), asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  push rax\n");
                    
                    codegen_expr_with_ctx(index, asm, string_map, 1, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  push rax\n");
                    
                    codegen_expr_with_ctx(value, asm, string_map, 2, label_counter, ctx, function_signatures, symbol_table);
                    
                    asm.push_str("  pop rdx\n");
                    asm.push_str("  pop rcx\n");
                    asm.push_str("  mov r8, rax\n");
                    
                    asm.push_str("  call .L_dict_insert\n");
                }
                _ => {
                    codegen_expr_with_ctx(&Expr::Var(list.clone()), asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  push rax\n");
                    
                    codegen_expr_with_ctx(index, asm, string_map, 1, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  push rax\n");
                    
                    codegen_expr_with_ctx(value, asm, string_map, 2, label_counter, ctx, function_signatures, symbol_table);
                    
                    asm.push_str("  pop r10\n");
                    asm.push_str("  pop r9\n");
                    
                    asm.push_str("  cmp r10, 0\n");
                    asm.push_str("  jl .L_bounds_error\n");
                    asm.push_str("  mov r11, [r9 + 8]\n");
                    asm.push_str("  cmp r10, r11\n");
                    asm.push_str("  jge .L_bounds_error\n");
                    
                    asm.push_str("  mov [r9 + 16 + 8 * r10], rax\n");
                }
            }
        }
        Stmt::Print(expr) => {
            codegen_expr_with_ctx(expr, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
            let expr_type = get_expr_type(expr, symbol_table, function_signatures, None);
            asm.push_str("  mov rdx, rax\n");
            match expr_type {
                Type::Int => {
                    asm.push_str("  mov rcx, fmt_int\n");
                }
                Type::Str => {
                    asm.push_str("  mov rcx, fmt_str\n");
                }
                _ => {
                    asm.push_str("  mov rcx, fmt_int\n");
                }
            }
            asm.push_str("  call [printf]\n");
        }
        Stmt::If { cond, then_branch } => {
            let label_idx = *label_counter;
            *label_counter += 1;
            codegen_expr_with_ctx(cond, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  cmp rax, 0\n");
            asm.push_str(&format!("  je .L_end_{}\n", label_idx));
            for s in then_branch {
                codegen_stmt_with_ctx(s, asm, string_map, symbol_table, function_signatures, label_counter, ctx);
            }
            asm.push_str(&format!(".L_end_{}:\n", label_idx));
        }
        Stmt::While { cond, body } => {
            let label_idx = *label_counter;
            *label_counter += 1;
            asm.push_str(&format!(".L_cond_{}:\n", label_idx));
            codegen_expr_with_ctx(cond, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  cmp rax, 0\n");
            asm.push_str(&format!("  je .L_end_{}\n", label_idx));
            for s in body {
                codegen_stmt_with_ctx(s, asm, string_map, symbol_table, function_signatures, label_counter, ctx);
            }
            asm.push_str(&format!("  jmp .L_cond_{}\n", label_idx));
            asm.push_str(&format!(".L_end_{}:\n", label_idx));
        }
        Stmt::FnDef { .. } => {}
        Stmt::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                codegen_expr_with_ctx(expr, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
            } else {
                asm.push_str("  mov rax, 0\n");
            }
            if let Some(c) = ctx {
                asm.push_str(&format!("  jmp .L_epilogue_{}\n", c.func_name));
            }
        }
        Stmt::Append { list, item } => {
            let label_idx = *label_counter;
            *label_counter += 1;
            
            codegen_expr_with_ctx(list, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  push rax\n");
            
            codegen_expr_with_ctx(item, asm, string_map, 1, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  pop r12\n");
            asm.push_str("  push rax\n");
            
            asm.push_str("  mov r10, [r12 + 8]\n");
            asm.push_str("  mov r11, [r12 + 0]\n");
            
            asm.push_str("  cmp r10, r11\n");
            asm.push_str(&format!("  jne .L_no_realloc_{}\n", label_idx));
            
            asm.push_str("  shl r11, 1\n");
            asm.push_str("  mov [r12 + 0], r11\n");
            asm.push_str("  imul r11, 8\n");
            asm.push_str("  add r11, 16\n");
            
            asm.push_str("  push r12\n");
            asm.push_str("  push r10\n");
            
            asm.push_str("  mov rdx, r11\n");
            asm.push_str("  mov rcx, r12\n");
            
            asm.push_str("  call [realloc]\n");
            
            asm.push_str("  pop r10\n");
            asm.push_str("  pop r12\n");
            asm.push_str("  mov r12, rax\n");
            
            if let Expr::Var(name) = list {
                if let Some(c) = ctx {
                    if let Some(offset) = c.local_offsets.get(name) {
                        asm.push_str(&format!("  mov [rbp {}], r12\n", if *offset >= 0 { format!("+ {}", offset) } else { format!("- {}", -offset) }));
                    } else {
                        asm.push_str(&format!("  mov [var_{}], r12\n", name));
                    }
                } else {
                    asm.push_str(&format!("  mov [var_{}], r12\n", name));
                }
            }
            
            asm.push_str(&format!(".L_no_realloc_{}:\n", label_idx));
            asm.push_str("  pop r13\n");
            asm.push_str("  mov [r12 + 16 + 8 * r10], r13\n");
            asm.push_str("  inc r10\n");
            asm.push_str("  mov [r12 + 8], r10\n");
        }
        Stmt::WriteFile { filename, content } => {
            codegen_expr_with_ctx(filename, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  mov r12, rax\n");
            
            codegen_expr_with_ctx(content, asm, string_map, 1, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  mov r13, rax\n");
            
            asm.push_str("  mov rdx, file_mode_w\n");
            asm.push_str("  mov rcx, r12\n");
            asm.push_str("  call [fopen]\n");
            
            asm.push_str("  test rax, rax\n");
            asm.push_str("  jz .L_file_error\n");
            asm.push_str("  mov r12, rax\n");
            
            asm.push_str("  mov rcx, r13\n");
            asm.push_str("  call [strlen]\n");
            
            asm.push_str("  mov r9, r12\n");
            asm.push_str("  mov r8, rax\n");
            asm.push_str("  mov rdx, 1\n");
            asm.push_str("  mov rcx, r13\n");
            asm.push_str("  call [fwrite]\n");
            
            asm.push_str("  mov rcx, r12\n");
            asm.push_str("  call [fclose]\n");
        }
        Stmt::TryCatch { try_block, catch_block } => {
            let label_idx = *label_counter;
            *label_counter += 1;
            
            asm.push_str("  mov rax, [exception_depth]\n");
            asm.push_str("  mov rcx, [bro_exception_handler]\n");
            asm.push_str("  mov [saved_handlers + 8 * rax], rcx\n");
            asm.push_str("  mov [saved_rsps + 8 * rax], rsp\n");
            asm.push_str("  inc rax\n");
            asm.push_str("  mov [exception_depth], rax\n");
            asm.push_str(&format!("  mov rcx, .L_catch_{}\n", label_idx));
            asm.push_str("  mov [bro_exception_handler], rcx\n");
            
            for s in try_block {
                codegen_stmt_with_ctx(s, asm, string_map, symbol_table, function_signatures, label_counter, ctx);
            }
            
            asm.push_str("  mov rax, [exception_depth]\n");
            asm.push_str("  dec rax\n");
            asm.push_str("  mov [exception_depth], rax\n");
            asm.push_str("  mov rcx, [saved_handlers + 8 * rax]\n");
            asm.push_str("  mov [bro_exception_handler], rcx\n");
            asm.push_str(&format!("  jmp .L_end_{}\n", label_idx));
            
            asm.push_str(&format!(".L_catch_{}:\n", label_idx));
            asm.push_str("  mov rax, [exception_depth]\n");
            asm.push_str("  dec rax\n");
            asm.push_str("  mov [exception_depth], rax\n");
            asm.push_str("  mov rsp, [saved_rsps + 8 * rax]\n");
            asm.push_str("  mov rcx, [saved_handlers + 8 * rax]\n");
            asm.push_str("  mov [bro_exception_handler], rcx\n");
            asm.push_str("  mov qword [bro_error_code], 0\n");
            
            for s in catch_block {
                codegen_stmt_with_ctx(s, asm, string_map, symbol_table, function_signatures, label_counter, ctx);
            }
            
            asm.push_str(&format!(".L_end_{}:\n", label_idx));
        }
        Stmt::Expr(expr) => {
            codegen_expr_with_ctx(expr, asm, string_map, 0, label_counter, ctx, function_signatures, symbol_table);
        }
    }
}

fn codegen_expr_with_ctx(
    expr: &Expr,
    asm: &mut String,
    string_map: &HashMap<String, usize>,
    depth: usize,
    label_counter: &mut usize,
    ctx: Option<&CodegenCtx>,
    function_signatures: &HashMap<String, (Vec<Type>, Type)>,
    symbol_table: &HashMap<String, VariableInfo>,
) {
    match expr {
        Expr::Int(val) => {
            asm.push_str(&format!("  mov rax, {}\n", val));
        }
        Expr::Str(val) => {
            let idx = string_map.get(val).unwrap();
            asm.push_str(&format!("  mov rax, str_lit_{}\n", idx));
        }
        Expr::Var(name) => {
            if let Some(c) = ctx {
                if let Some(offset) = c.local_offsets.get(name) {
                    asm.push_str(&format!("  mov rax, [rbp {}]\n", if *offset >= 0 { format!("+ {}", offset) } else { format!("- {}", -offset) }));
                } else {
                    asm.push_str(&format!("  mov rax, [var_{}]\n", name));
                }
            } else {
                asm.push_str(&format!("  mov rax, [var_{}]\n", name));
            }
        }
        Expr::Input { id } => {
            asm.push_str(&format!("  mov rdx, input_buf_{}\n", id));
            asm.push_str("  mov rcx, fmt_str_in\n");
            asm.push_str("  call [scanf]\n");
            asm.push_str(&format!("  mov rax, input_buf_{}\n", id));
        }
        Expr::Len(sub) => {
            codegen_expr_with_ctx(sub, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  mov rcx, rax\n");
            asm.push_str("  call [strlen]\n");
        }
        Expr::Sleep(sub) => {
            codegen_expr_with_ctx(sub, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  mov rcx, rax\n");
            asm.push_str("  call [Sleep]\n");
            asm.push_str("  mov rax, 0\n");
        }
        Expr::Random => {
            asm.push_str("  call [rand]\n");
        }
        Expr::Alert { title, message } => {
            codegen_expr_with_ctx(title, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  push rax\n");
            codegen_expr_with_ctx(message, asm, string_map, depth + 1, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  pop r10\n");
            
            asm.push_str("  mov rdx, rax\n");
            asm.push_str("  mov r8, r10\n");
            asm.push_str("  mov rcx, 0\n");
            asm.push_str("  mov r9, 0\n");
            asm.push_str("  call [MessageBoxA]\n");
        }
        Expr::Window { title, width, height } => {
            codegen_expr_with_ctx(title, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  push rax\n");
            codegen_expr_with_ctx(width, asm, string_map, depth + 1, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  push rax\n");
            codegen_expr_with_ctx(height, asm, string_map, depth + 2, label_counter, ctx, function_signatures, symbol_table);
            
            asm.push_str("  pop r11\n");
            asm.push_str("  pop r10\n");
            
            asm.push_str("  mov qword [rsp + 32], 0x80000000\n");
            asm.push_str("  mov qword [rsp + 40], 0x80000000\n");
            asm.push_str("  mov [rsp + 48], r11\n");
            asm.push_str("  mov [rsp + 56], rax\n");
            asm.push_str("  mov qword [rsp + 64], 0\n");
            asm.push_str("  mov qword [rsp + 72], 0\n");
            asm.push_str("  mov qword [rsp + 80], 0\n");
            asm.push_str("  mov qword [rsp + 88], 0\n");
            
            asm.push_str("  mov rcx, 0\n");
            asm.push_str("  mov rdx, window_class_name\n");
            asm.push_str("  mov r8, r10\n");
            asm.push_str("  mov r9, 0x10CF0000\n");
            
            asm.push_str("  call [CreateWindowExA]\n");
            
            let label_idx = *label_counter;
            *label_counter += 1;
            asm.push_str(&format!(".L_msg_loop_{}:\n", label_idx));
            asm.push_str("  mov rdx, 0\n");
            asm.push_str("  mov r8, 0\n");
            asm.push_str("  mov r9, 0\n");
            asm.push_str("  mov rcx, msg_struct\n");
            asm.push_str("  call [GetMessageA]\n");
            asm.push_str("  cmp rax, 0\n");
            asm.push_str(&format!("  je .L_loop_end_{}\n", label_idx));
            asm.push_str("  mov rcx, msg_struct\n");
            asm.push_str("  call [TranslateMessage]\n");
            asm.push_str("  mov rcx, msg_struct\n");
            asm.push_str("  call [DispatchMessageA]\n");
            asm.push_str(&format!("  jmp .L_msg_loop_{}\n", label_idx));
            asm.push_str(&format!(".L_loop_end_{}:\n", label_idx));
            
            asm.push_str("  mov rax, 0\n");
        }
        Expr::Call { name, args } => {
            let num_args = args.len();
            for i in 0..num_args {
                codegen_expr_with_ctx(&args[i], asm, string_map, depth + i, label_counter, ctx, function_signatures, symbol_table);
                if i < 4 {
                    asm.push_str("  push rax\n");
                } else {
                    let offset = 32 + 8 * (i - 4);
                    asm.push_str(&format!("  mov [rsp + {}], rax\n", offset + 32));
                }
            }
            for idx in (0..std::cmp::min(num_args, 4)).rev() {
                let reg = match idx {
                    0 => "rcx",
                    1 => "rdx",
                    2 => "r8",
                    3 => "r9",
                    _ => unreachable!(),
                };
                asm.push_str(&format!("  pop {}\n", reg));
            }
            asm.push_str(&format!("  call fn_{}\n", name));
        }
        Expr::ListLiteral(elements) => {
            let capacity = std::cmp::max(elements.len(), 4);
            let bytes = capacity * 8 + 16;
            
            asm.push_str(&format!("  mov rcx, {}\n", bytes));
            asm.push_str("  call [malloc]\n");
            
            asm.push_str(&format!("  mov qword [rax + 0], {}\n", capacity));
            asm.push_str(&format!("  mov qword [rax + 8], {}\n", elements.len()));
            
            if !elements.is_empty() {
                asm.push_str("  push rax\n");
                for (i, el) in elements.iter().enumerate() {
                    codegen_expr_with_ctx(el, asm, string_map, depth + 1, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  pop r10\n");
                    asm.push_str(&format!("  mov [r10 + 16 + 8 * {}], rax\n", i));
                    asm.push_str("  push r10\n");
                }
                asm.push_str("  pop rax\n");
            }
        }
        Expr::DictLiteral(entries) => {
            asm.push_str("  mov rcx, 128\n");
            asm.push_str("  call [malloc]\n");
            
            asm.push_str("  push rax\n");
            asm.push_str("  mov rcx, 16\n");
            asm.push_str("  mov rdi, rax\n");
            asm.push_str("  xor rax, rax\n");
            asm.push_str("  rep stosq\n");
            asm.push_str("  pop rax\n");
            
            if !entries.is_empty() {
                asm.push_str("  push rax\n");
                for entry in entries {
                    codegen_expr_with_ctx(&entry.0, asm, string_map, depth + 1, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  push rax\n");
                    
                    codegen_expr_with_ctx(&entry.1, asm, string_map, depth + 2, label_counter, ctx, function_signatures, symbol_table);
                    
                    asm.push_str("  pop rdx\n");
                    asm.push_str("  pop rcx\n");
                    asm.push_str("  mov r8, rax\n");
                    
                    asm.push_str("  push rcx\n");
                    asm.push_str("  call .L_dict_insert\n");
                }
                asm.push_str("  pop rax\n");
            }
        }
        Expr::ListIndex { list, index } => {
            let list_type = get_expr_type(list, symbol_table, function_signatures, None);
            match list_type {
                Type::Dict(..) => {
                    codegen_expr_with_ctx(list, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  push rax\n");
                    
                    codegen_expr_with_ctx(index, asm, string_map, depth + 1, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  pop rcx\n");
                    asm.push_str("  mov rdx, rax\n");
                    
                    asm.push_str("  call .L_dict_lookup\n");
                }
                _ => {
                    codegen_expr_with_ctx(list, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  push rax\n");
                    codegen_expr_with_ctx(index, asm, string_map, depth + 1, label_counter, ctx, function_signatures, symbol_table);
                    asm.push_str("  pop r10\n");
                    
                    asm.push_str("  cmp rax, 0\n");
                    asm.push_str("  jl .L_bounds_error\n");
                    asm.push_str("  mov r11, [r10 + 8]\n");
                    asm.push_str("  cmp rax, r11\n");
                    asm.push_str("  jge .L_bounds_error\n");
                    
                    asm.push_str("  mov rax, [r10 + 16 + 8 * rax]\n");
                }
            }
        }
        Expr::ReadFile(sub) => {
            codegen_expr_with_ctx(sub, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  mov r15, rax\n"); // Save filename pointer to non-volatile R15
            
            asm.push_str("  mov rdx, file_mode_rb\n");
            asm.push_str("  mov rcx, r15\n");
            asm.push_str("  call [fopen]\n");
            
            asm.push_str("  test rax, rax\n");
            asm.push_str("  jz .L_file_error\n");
            asm.push_str("  mov r12, rax\n");
            
            asm.push_str("  mov r8, 2\n");
            asm.push_str("  mov rdx, 0\n");
            asm.push_str("  mov rcx, r12\n");
            asm.push_str("  call [fseek]\n");
            
            asm.push_str("  mov rcx, r12\n");
            asm.push_str("  call [ftell]\n");
            asm.push_str("  mov r13, rax\n");
            
            asm.push_str("  mov r8, 0\n");
            asm.push_str("  mov rdx, 0\n");
            asm.push_str("  mov rcx, r12\n");
            asm.push_str("  call [fseek]\n");
            
            asm.push_str("  mov rcx, r13\n");
            asm.push_str("  inc rcx\n");
            asm.push_str("  call [malloc]\n");
            asm.push_str("  mov r14, rax\n");
            
            asm.push_str("  mov r9, r12\n");
            asm.push_str("  mov r8, r13\n");
            asm.push_str("  mov rdx, 1\n");
            asm.push_str("  mov rcx, r14\n");
            asm.push_str("  call [fread]\n");
            
            asm.push_str("  mov byte [r14 + r13], 0\n");
            
            asm.push_str("  mov rcx, r12\n");
            asm.push_str("  call [fclose]\n");
            
            asm.push_str("  mov rax, r14\n");
        }
        Expr::Binary { op, left, right } => {
            codegen_expr_with_ctx(left, asm, string_map, depth, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  push rax\n");
            codegen_expr_with_ctx(right, asm, string_map, depth + 1, label_counter, ctx, function_signatures, symbol_table);
            asm.push_str("  pop r10\n");
            
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
                    asm.push_str("  mov r11, rax\n");
                    asm.push_str("  mov rax, r10\n");
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
