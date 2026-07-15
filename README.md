# BroLang Programming Language

**BroLang** is a bilingual, typesafe, compiled programming language designed to make native systems programming and compiler engineering highly accessible to beginners. It compiles directly into standard 64-bit Windows x64 assembly (Flat Assembler - FASM) and translates statements into native machine code.

---

## Key Features

1. **Bilingual Keywords**: Swap between English and German keywords seamlessly. All compiler parts support interchangeable syntax equivalents.
2. **Semicolon-Free & End Closures**: Statements require no trailing semicolons. Structuring closures end with `end` or `ende`.
3. **Recursive Functions**: Full support for recursive function invocations (`fn`/`funktion`) using stack-relative calling frames and parameter bindings.
4. **Target-Oriented Type Inference**: Resolves ambiguous types (such as `input()`) based on target assignment expressions. Emits strict compile-time errors for type mismatches.
5. **Elm/Rust-Style Diagnostics**: Highlights the exact line, column, and span of parse/type errors, accompanied by friendly contextual suggestions.
6. **Windows GUI & System Integration**: Direct bindings to `USER32.DLL`, `KERNEL32.DLL`, and `MSVCRT.DLL` to spawn graphical windows, trigger message box alerts, read input, and implement delays.
7. **Clean FASM Assembly Generation**: Implements 16-byte stack frame alignment, shadow space reservation, and function namespace protection (`fn_`) to prevent collisions with assembly opcodes.

---

## Bilingual Keyword Reference

| Feature | English Keyword | German Keyword | FASM / Windows ABI Execution |
| :--- | :--- | :--- | :--- |
| **Declaration** | `set [var] to [val]` | `setze [var] auf [val]` | Stack/data relative value binding |
| **Output** | `print [expr]` / `show([expr])` | `zeige [expr]` / `zeige([expr])` | `printf` (`MSVCRT.DLL`) |
| **Conditionals** | `if [cond] ... end` | `wenn [cond] ... ende` | Compare and Jump labels |
| **Looping** | `while [cond] ... end` | `solange [cond] ... ende` | Jump-based looping |
| **Function Definition**| `fn [name](params) ... end`| `funktion [name](params) ... ende`| Assembly procedure blocks (`fn_name`) |
| **Returns** | `return [expr]` | `rueckgabe [expr]` / `zurueck` | Value returned in `rax` register |
| **Input** | `input()` | `lese()` | `scanf` (`MSVCRT.DLL`) |
| **Length** | `len(str)` | `laenge(str)` | `strlen` (`MSVCRT.DLL`) |
| **Delay** | `sleep(ms)` | `warte(ms)` | `Sleep` (`KERNEL32.DLL`) |
| **Random** | `random()` | `zufall()` | `rand` (`MSVCRT.DLL`) |
| **Alert Box** | `alert(title, msg)` | `info(titel, nachricht)` | `MessageBoxA` (`USER32.DLL`) |
| **Windows GUI** | `window(t, w, h)` | `fenster(t, w, h)` | `CreateWindowExA` & loop (`USER32.DLL`)|

---

## Global CLI Installation

BroLang comes with an automated PowerShell installer script that sets up the compiler and assembler globally on your system.

### 1. Run the Installer
Open PowerShell inside the repository directory and run the installer script:
```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

**What the installer does:**
1. Compiles the compiler in release mode (`cargo build --release`).
2. Creates a user-local directory at `C:\Users\<YourUser>\.brolang\bin`.
3. Copies the compiled compiler `bro.exe` to that folder.
4. Downloads the official Flat Assembler zip, extracts the compiler executable, and copies/renames it to `fasm2.exe` in the same directory.
5. Adds the bin folder to the current user's registry `PATH` variable.

### 2. Reload Environment Path (Optional)
To use the `bro` command immediately in your *current* terminal session without restarting, reload the environment path:
```powershell
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

### 3. Verify the Installation
Run:
```bash
bro --version
```
Output:
```text
BroLang Compiler v1.0.0
```

---

## Compilation Usage

Compile and run any `.bro` script from any folder in your terminal:
```bash
bro my_program.bro
```
This automatically parses, type-checks, generates `output.asm`, and invokes the bundled `fasm2.exe` to compile a standalone, native, high-performance Windows executable `output.exe`!

---

## Code Examples

### 1. English Recursive Fibonacci
```python
# Compute the 6th Fibonacci number
fn fib(n)
    if n <= 1
        return n
    end
    set a to fib(n - 1)
    set b to fib(n - 2)
    return a + b
end

print fib(6)
```

### 2. German Graphical Window and Pop-up
```python
# Spawns a native Windows Alert Info Box
info("BroLang Info", "Gleich oeffnet sich ein 800x600 Fenster!")

# Spawns a graphic frame. Clicking "X" exits the program cleanly.
fenster("Mein BroLang Fenster", 800, 600)
```

---

## Compiler Design Deep-Dive

### 1. Recursive Stack Frame Layout
To support recursive function executions, variables are stored locally on the stack rather than globally.
* **Procedures**: Each `fn`/`funktion` compiles to a FASM label prefixed with `fn_` (e.g. `fn_fib`) to prevent collisions with FASM opcodes (such as `add`).
* **Frame Pointer**: The function prologue sets up the frame pointer (`rbp`):
  ```assembly
  push rbp
  mov rbp, rsp
  sub rsp, 48   # Allocates local variables + shadow space, maintaining 16-byte alignment
  ```
* **Parameters**: Copied from registers to caller shadow space relative to `rbp`:
  * Parameter 1 (`RCX`) -> `[rbp + 16]`
  * Parameter 2 (`RDX`) -> `[rbp + 24]`
  * Parameter 3 (`R8`)  -> `[rbp + 32]`
  * Parameter 4 (`R9`)  -> `[rbp + 40]`
  * Parameters 5+ are read from stack offsets `[rbp + 48]`, `[rbp + 56]`, etc.
* **Locals**: Function-scoped local variables are allocated negative RBP offsets, e.g. `[rbp - 8]`, `[rbp - 16]`.

### 2. 16-byte Stack Alignment
Before invoking any external Win32/MSVCRT function, the Windows x64 ABI requires the stack pointer (`RSP`) to be aligned to a 16-byte boundary. The compiler tracks stack operations (pushes/pops) dynamically:
* If the stack depth is odd, it pads with `sub rsp, 8` before function `call` instructions, and restores it with `add rsp, 8` after return.
* Win32 event loops are fully optimized and stack-padded during window callbacks.
