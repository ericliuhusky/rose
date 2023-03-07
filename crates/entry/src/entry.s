    .section .text.entry
    .globl _start
_start:
    la sp, __BOOT_STACK_TOP
    call main

    .section .data
__BOOT_STACK:
    .space 0x10000
__BOOT_STACK_TOP:
