# BroLang Programming Language

**BroLang** is a bilingual, typesafe, compiled programming language designed to make native systems programming and compiler engineering highly accessible to beginners. It compiles directly into standard 64-bit Windows x64 assembly (Flat Assembler - FASM 2) and translates statements into native machine code.

---

## Key Features

1. **Bilingual Keywords**: Swap between English and German keywords seamlessly. All compiler parts support interchangeable syntax equivalents.
2. **Semicolon-Free & End Closures**: Statements require no trailing semicolons. Structuring closures end with `end` or `ende`.
3. **Recursive Functions**: Full support for recursive function definitions (`fn`/`funktion`) using stack-relative calling frames and parameter bindings.
4. **Local Scoping (Stack Frames)**: Standard local scope management. Local variables inside functions reside on the stack frame of the active function and do not leak into the global scope.
5. **Dynamic Arrays / Lists**: Heap-allocated arrays (`malloc`/`realloc` under the hood) supporting indexed lookups and dynamic appending (`append`/`anfuegen`).
6. **Dictionaries (Heap Hashmaps)**: Heap-allocated key-value maps. Implements an inline 64-bit FNV-1a hashing algorithm in assembly, using linked list chaining to resolve hash collisions. Supports heterogenous value mappings (mixed strings and integers).
7. **High-Level File I/O**: Direct stream utilities to write to files (`write_file`/`schreibe_datei`) and read whole files into heap-allocated strings (`read_file`/`lese_datei`).
8. **Structured Exception Handling (Try-Catch)**: Robust try-catch blocks (`try-catch-end` / `versuche-fange-ende`) to catch runtime failures (Array Bounds Violation, File Open Failure, Dictionary Key Missing) without crashing.
9. **Clean FASM Assembly Generation**: Enforces strict 16-byte stack alignment using standard `and rsp, -16` function prologues, ensuring safe, crash-free invocations of SSE-optimized C-Runtime (MSVCRT.DLL) and Win32 functions.

---

## Bilingual Keyword Reference

| Feature | English Keyword | German Keyword | FASM / Windows ABI Execution |
| :--- | :--- | :--- | :--- |
| **Declaration** | `set [var] to [val]` | `setze [var] auf [val]` | Stack/data relative value binding |
| **Index Assignment** | `set [dict][key] to [val]` | `setze [dict][key] auf [val]` | Dictionary insert/update operation |
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
| **List Append** | `append(list, item)` | `anfuegen(list, item)` | `realloc` allocation & element append |
| **File Writing** | `write_file(path, data)` | `schreibe_datei(pfad, daten)` | `fopen` ("w"), `fwrite`, `fclose` (`MSVCRT.DLL`) |
| **File Reading** | `read_file(path)` | `lese_datei(pfad)` | `fopen` ("rb"), `malloc`, `fread`, `fclose` |
| **Exception Handling** | `try ... catch ... end` | `versuche ... fange ... ende` | Exception handler routing & stack restoration |

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

---

## Compilation Usage

Compile and run any `.bro` script from any folder in your terminal:
```bash
bro my_program.bro
```
This automatically parses, type-checks, generates `output.asm`, and invokes the bundled `fasm2.exe` to compile a standalone, native, high-performance Windows executable `output.exe`!

---

## Code Examples

### 1. Recursive Fibonacci with Scope Isolation (English)
```python
# Compute the 6th Fibonacci number using isolated stack frames
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

### 2. Dynamic Heap Lists & Index Out-of-Bounds Exception (English)
```python
# Allocates array on the heap
set my_list to [10, 20, 30]
print my_list[1] # Prints 20

# Appends items dynamically, resizing the array (realloc)
append(my_list, 40)
print my_list[3] # Prints 40

# Structured safety check
try
    # Out of bounds index access would crash without try-catch!
    print my_list[10]
catch
    print "Recovered from index out of bounds error!"
end
```

### 3. Heterogenous Dictionaries, File I/O & Exceptions (German)
```python
# Create a dictionary mapping keys to mixed string/integer values
setze benutzer auf {"name": "Jona", "alter": 15}
zeige benutzer["name"] # Zeigt "Jona"
zeige benutzer["alter"] # Zeigt 15

# Write and read back from a file
schreibe_datei("test.txt", "BroLang Datei-Schreiben funktioniert!")
setze inhalt auf lese_datei("test.txt")
zeige inhalt

# Catch dynamic runtime errors
versuche
    # If file doesn't exist, we jump to catch block instead of crashing
    setze inhalt_alt auf lese_datei("ungueltig.txt")
fange
    zeige "Datei konnte nicht geoeffnet werden!"
ende
```

---

## Compiler Design Deep-Dive

### 1. 16-byte Stack Alignment Prologue
To conform to the 64-bit Windows Calling Convention, the stack pointer (`RSP`) must be aligned to a 16-byte boundary before calling any DLL function. The BroLang compiler enforces this at the entry of the main thread and every function body:
```assembly
start:
  push rbp
  mov rbp, rsp
  and rsp, -16  ; Enforce 16-byte alignment
  sub rsp, 32   ; Reserve Win64 shadow space
```
Because `RSP` is strictly aligned and stays aligned throughout calculations, calls to imports like `printf`, `malloc`, or `fopen` can be executed directly without inline alignment corrections.

### 2. Dynamic List Memory Layout
A dynamic list variable holds a pointer to a heap-allocated buffer containing:
* `[ptr + 0]`: Capacity (64-bit integer representing max pre-allocated slots).
* `[ptr + 8]`: Length (64-bit integer representing occupied slots).
* `[ptr + 16]`: Elements (sequentially stored 64-bit values).
During appends, the compiler compares `Length` and `Capacity`. If equal, it doubles `Capacity` and calls `realloc` before writing the item.

### 3. FNV-1a Hashed Dictionaries
A dictionary holds a pointer to a 128-byte array representing 16 bucket pointers (hash indices).
1. **Hashing**: The key string is hashed using the 64-bit FNV-1a algorithm:
   $$hash = (hash \oplus byte) \times 0x100000001b3$$
2. **Buckets**: The bucket index is computed as `hash & 15`.
3. **Collision Chains**: Each bucket head points to a linked list of entries, where each entry node is a 24-byte struct:
   * `[node + 0]`: Key string pointer.
   * `[node + 8]`: Value payload (64-bit integer or string pointer).
   * `[node + 16]`: Next entry pointer (0 if end of chain).

### 4. Structured Exceptions Stack Recovery
When entering a `try` block, the compiler pushes context variables to a global recovery area (`saved_rsps` and `saved_handlers`) and registers the `catch` block address in a thread-local variable `bro_exception_handler`.
* If a bounds check fails, a file open returns NULL, or a hashmap lookup fails:
  * The runtime checks if `[bro_exception_handler]` is active.
  * If yes, it loads the saved `RSP` to restore the stack frame, restores the parent exception handler, and jumps directly to the `catch` label.
  * If no, it prints a diagnostic error and calls `ExitProcess(1)`.
