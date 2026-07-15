format PE64 console
entry start

section '.data' data readable writeable
  fmt_int db '%lld', 13, 10, 0
  fmt_str db '%s', 13, 10, 0
  fmt_int_in db '%lld', 0
  fmt_str_in db '%s', 0
  str_lit_2 db 'Mein BroLang Fenster', 0
  str_lit_0 db 'BroLang Info', 0
  str_lit_1 db 'Hallo aus der Windows-GUI!', 0
  window_class_name db 'BroLangWndClass', 0
  msg_struct rb 48
  wnd_class:
    wc_style         dd 0
                     dd 0 ; alignment
    wc_lpfnWndProc   dq rva window_proc
    wc_cbClsExtra    dd 0
    wc_cbWndExtra    dd 0
    wc_hInstance     dq 0
    wc_hIcon         dq 0
    wc_hCursor       dq 0
    wc_hbrBackground dq 6
    wc_lpszMenuName  dq 0
    wc_lpszClassName dq window_class_name

section '.text' code readable executable
start:
  sub rsp, 104

  ; Register Win32 Class
  mov rcx, wnd_class
  call [RegisterClassA]

  mov rax, str_lit_0
  push rax
  mov rax, str_lit_1
  pop r10
  mov rdx, rax
  mov r8, r10
  mov rcx, 0
  mov r9, 0
  call [MessageBoxA]
  mov rax, str_lit_2
  push rax
  mov rax, 800
  push rax
  mov rax, 600
  pop r11
  pop r10
  mov qword [rsp + 32], 0x80000000
  mov qword [rsp + 40], 0x80000000
  mov [rsp + 48], r11
  mov [rsp + 56], rax
  mov qword [rsp + 64], 0
  mov qword [rsp + 72], 0
  mov qword [rsp + 80], 0
  mov qword [rsp + 88], 0
  mov rcx, 0
  mov rdx, window_class_name
  mov r8, r10
  mov r9, 0x10CF0000
  call [CreateWindowExA]
.L_msg_loop_0:
  mov rdx, 0
  mov r8, 0
  mov r9, 0
  mov rcx, msg_struct
  call [GetMessageA]
  cmp rax, 0
  je .L_loop_end_0
  mov rcx, msg_struct
  call [TranslateMessage]
  mov rcx, msg_struct
  call [DispatchMessageA]
  jmp .L_msg_loop_0
.L_loop_end_0:
  mov rax, 0
  mov rcx, 0
  call [ExitProcess]

window_proc:
  cmp rdx, 2 ; WM_DESTROY
  je .L_destroy_wnd
  sub rsp, 40
  call [DefWindowProcA]
  add rsp, 40
  ret
.L_destroy_wnd:
  mov rcx, 0
  call [ExitProcess]

section '.idata' import data readable

  dd rva kernel32_lookup, 0, 0, rva kernel32_name, rva kernel32_address
  dd rva msvcrt_lookup, 0, 0, rva msvcrt_name, rva msvcrt_address
  dd rva user32_lookup, 0, 0, rva user32_name, rva user32_address
  dd 0, 0, 0, 0, 0

  kernel32_lookup:
    dq rva kernel32_ExitProcess
    dq rva kernel32_Sleep
    dq 0

  kernel32_address:
    ExitProcess dq rva kernel32_ExitProcess
    Sleep       dq rva kernel32_Sleep
    dq 0

  msvcrt_lookup:
    dq rva msvcrt_printf
    dq rva msvcrt_scanf
    dq rva msvcrt_strlen
    dq rva msvcrt_rand
    dq 0

  msvcrt_address:
    printf      dq rva msvcrt_printf
    scanf       dq rva msvcrt_scanf
    strlen      dq rva msvcrt_strlen
    rand        dq rva msvcrt_rand
    dq 0

  user32_lookup:
    dq rva user32_MessageBoxA
    dq rva user32_RegisterClassA
    dq rva user32_CreateWindowExA
    dq rva user32_DefWindowProcA
    dq rva user32_GetMessageA
    dq rva user32_TranslateMessage
    dq rva user32_DispatchMessageA
    dq 0

  user32_address:
    MessageBoxA      dq rva user32_MessageBoxA
    RegisterClassA   dq rva user32_RegisterClassA
    CreateWindowExA  dq rva user32_CreateWindowExA
    DefWindowProcA   dq rva user32_DefWindowProcA
    GetMessageA      dq rva user32_GetMessageA
    TranslateMessage dq rva user32_TranslateMessage
    DispatchMessageA dq rva user32_DispatchMessageA
    dq 0

  kernel32_name db 'KERNEL32.DLL', 0
  msvcrt_name   db 'MSVCRT.DLL', 0
  user32_name   db 'USER32.DLL', 0

  kernel32_ExitProcess dw 0
                       db 'ExitProcess', 0
  kernel32_Sleep       dw 0
                       db 'Sleep', 0

  msvcrt_printf        dw 0
                       db 'printf', 0
  msvcrt_scanf         dw 0
                       db 'scanf', 0
  msvcrt_strlen        dw 0
                       db 'strlen', 0
  msvcrt_rand          dw 0
                       db 'rand', 0

  user32_MessageBoxA      dw 0
                          db 'MessageBoxA', 0
  user32_RegisterClassA   dw 0
                          db 'RegisterClassA', 0
  user32_CreateWindowExA  dw 0
                          db 'CreateWindowExA', 0
  user32_DefWindowProcA   dw 0
                          db 'DefWindowProcA', 0
  user32_GetMessageA      dw 0
                          db 'GetMessageA', 0
  user32_TranslateMessage dw 0
                          db 'TranslateMessage', 0
  user32_DispatchMessageA dw 0
                          db 'DispatchMessageA', 0
