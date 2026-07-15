mod lexer;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("\x1b[1;36mBroLang Compiler v1.0.0\x1b[0m");
        eprintln!("Usage: {} <input_file.bro>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    
    // Read input file
    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[1;31mIO Error:\x1b[0m Could not read input file '{}'.", input_path);
            eprintln!("Details: {}", e);
            std::process::exit(1);
        }
    };

    // Split source into lines for error reporting
    let source_lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();

    // 1. Lexical Analysis
    let mut lex = lexer::Lexer::new(&source);
    let tokens = match lex.tokenize() {
        Ok(t) => t,
        Err(err) => {
            print_lex_error(&err, &source_lines);
            std::process::exit(1);
        }
    };

    // 2. Syntactic Analysis
    let program = match parser::parse_program(&tokens, &source_lines) {
        Ok(p) => p,
        Err(err) => {
            parser::print_parse_error(&err, &source_lines);
            std::process::exit(1);
        }
    };

    // 3. Static Semantic Analysis (Type Checking)
    let symbol_table = match codegen::type_check(&program) {
        Ok(st) => st,
        Err(err) => {
            codegen::print_type_error(&err, &source_lines);
            std::process::exit(1);
        }
    };

    // 4. Code Generation
    let assembly = codegen::generate_assembly(&program, &symbol_table);

    // Write ASM file
    let asm_path = "output.asm";
    if let Err(e) = fs::write(asm_path, &assembly) {
        eprintln!("\x1b[1;31mIO Error:\x1b[0m Could not write assembly file '{}'.", asm_path);
        eprintln!("Details: {}", e);
        std::process::exit(1);
    }
    println!("\x1b[1;32mSuccess:\x1b[0m Generated 64-bit assembly: \x1b[1m{}\x1b[0m", asm_path);

    // 5. Invoke FASM 2 Assembler
    let exe_path = "output.exe";
    println!("Invoking fasm2 compiler to produce executable...");
    match Command::new("fasm2")
        .arg(asm_path)
        .arg(exe_path)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("\x1b[1;32mCompilation successful!\x1b[0m Created native executable: \x1b[1m{}\x1b[0m", exe_path);
            } else {
                eprintln!("\x1b[1;31mAssembler Error:\x1b[0m fasm2 returned a non-zero exit code.");
                eprintln!("--- FASM 2 Stderr ---");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                eprintln!("--- FASM 2 Stdout ---");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("\x1b[1;33mWarning:\x1b[0m Assembly file written successfully, but the 'fasm2' binary could not be executed.");
            eprintln!("System Error: {}", e);
            eprintln!();
            eprintln!("\x1b[1;32mSuggestion:\x1b[0m");
            eprintln!("1. Make sure you have Flat Assembler 2 (fasm2) installed.");
            eprintln!("2. Ensure 'fasm2' is added to your environment's PATH variables.");
            eprintln!("3. You can compile the assembly manually using: fasm2 {} {}", asm_path, exe_path);
        }
    }
}

fn print_lex_error(err: &lexer::LexError, source_lines: &[String]) {
    eprintln!("\x1b[1;31mLex Error:\x1b[0m {}", err.message);
    if err.line > 0 && err.line <= source_lines.len() {
        eprintln!("At line {}, column {}:", err.line, err.column);
        eprintln!();
        let line_content = &source_lines[err.line - 1];
        eprintln!("  {:3} | {}", err.line, line_content);
        let padding = " ".repeat(err.column - 1);
        eprintln!("      | \x1b[1;31m{}^\x1b[0m", padding);
    }
    eprintln!();
}
