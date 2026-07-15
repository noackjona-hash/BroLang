# BroLang Programming Language

**BroLang** is a highly accessible, bilingual (English & German), compiled programming language designed specifically for beginners. It features a clean syntax, requires no semicolons, compiles to native high-performance machine code, and provides friendly, Rust/Elm-inspired error diagnostics.

---

## Key Features

* **Bilingual Syntax**: Write code in either English or German. Keyword equivalents are recognized interchangeably.
* **No Semicolons**: Clean, modern statement layouts.
* **Block Closures**: Structured blocks (conditionals, loops) terminate using `end` / `ende`.
* **Static Type Checking**: Prevents variable type mismatches at compile time with helpful diagnostic suggestions.
* **Friendly Diagnostics**: Rust/Elm-inspired error reports that print the exact source lines, point directly to the column of the error, and suggest fixes.
* **Headerless Win64 ABI Native Assembly**: Generates raw PE64 Flat Assembler 2 (FASM 2) assembly code, automatically maintaining strict 16-byte stack alignments for nested expressions and calling convention routines.
* **Built-in Systems & GUI Pop-ups**: Map directly to standard Windows DLL library functions (`MSVCRT.DLL`, `KERNEL32.DLL`, `USER32.DLL`) without any external dependencies.

---

## Syntax & Keywords Reference

| Feature | English Syntax | German Syntax | Under-the-hood System Call |
| :--- | :--- | :--- | :--- |
| **Assignment** | `set [var] to [expr]` | `setze [var] auf [expr]` | Variable allocation (`dq`) |
| **Output** | `print [expr]` / `show([expr])` | `zeige [expr]` / `zeige([expr])` | `printf` from `MSVCRT.DLL` |
| **Conditionals** | `if [expr] ... end` | `wenn [expr] ... ende` | Jump instructions (`je`/`jne`) |
| **Loops** | `while [expr] ... end` | `solange [expr] ... ende` | Jump and Loop logic |
| **Input** | `set name to input()` | `setze name auf lese()` | `scanf` from `MSVCRT.DLL` (reads `%s` or `%lld` contextually) |
| **Length** | `len(my_string)` | `laenge(my_string)` | `strlen` from `MSVCRT.DLL` |
| **Sleep** | `sleep(1500)` | `warte(500)` | `Sleep` from `KERNEL32.DLL` |
| **Random** | `set r to random()` | `setze r auf zufall()` | `rand` from `MSVCRT.DLL` |
| **Alert Popup** | `alert("Title", "Message")` | `info("Titel", "Nachricht")` | `MessageBoxA` from `USER32.DLL` |
| **Window Frame** | `window("Title", 800, 600)` | `fenster("Titel", 800, 600)` | `CreateWindowExA` & Event message loops from `USER32.DLL` |

---

## Codebase Architecture

The compiler is structured as a modular Rust binary crate:

* **[src/lexer.rs](file:///c:/projects/language/src/lexer.rs)**: Tokenizes source files, tracking precise `line`, `column`, and `length` coordinates for every single token.
* **[src/parser.rs](file:///c:/projects/language/src/parser.rs)**: Parses the token stream into an Abstract Syntax Tree (AST), handling mathematical operator precedence and emitting Elm-inspired syntax errors.
* **[src/codegen.rs](file:///c:/projects/language/src/codegen.rs)**: Performs semantic type checking (mapping target types and variable definitions) and generates self-contained raw PE64 assembly code.
* **[src/main.rs](file:///c:/projects/language/src/main.rs)**: Serves as the CLI orchestration driver. Writes generated assembly to `output.asm` and automatically invokes `fasm2` to produce a standalone executable (`output.exe`).

---

## Getting Started

### 1. Build the Compiler
Build the Rust project using Cargo:
```bash
cargo build --release
```

### 2. Compile a BroLang Program
Compile any BroLang script (e.g. English, German, or GUI programs):
```bash
cargo run -- test/test_nest.bro
```
This generates `output.asm` and attempts to compile it into `output.exe` if `fasm2` is available on the system path.

### 3. Assemble Manually
If FASM 2 is not on your environment's path, assemble the generated `output.asm` manually:
```bash
fasm2 output.asm output.exe
```

---

## Code Examples

### English Counter & Input Nesting
```python
# Read name and print hello
print "Enter your name:"
set name to input()
print "Hello,"
print name

# Loop countdown
set counter to 5
while counter > 0
  print counter
  set counter to counter - 1
end
```

### German GUI alert & Window
```python
# Show an information box
info("BroLang Info", "Gleich oeffnet sich ein Fenster!")

# Create a visible Win32 window (blocks until closed)
fenster("Mein BroLang Fenster", 800, 600)
```
