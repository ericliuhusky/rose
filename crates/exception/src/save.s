.altmacro
.macro SAVE_REG n
    sd x\n, \n*8(sp)
.endm

    .section .text.trampoline
    .globl __save
__save:
    # 将用户栈指针暂存在sscratch
    csrw sscratch, sp


    # 用户栈指针已经暂存在sscratch，可以使用sp寄存器；其它寄存器还未保存，不可以使用其它寄存器
    ld sp, CONTEXT_START_ADDR

    # 保存除sp(x2)外的其它寄存器到上下文
    sd x1, 1*8(sp)
    .set n, 3
    .rept 29
        SAVE_REG %n
        .set n, n+1
    .endr


    # 栈指针指向内核栈
    ld sp, KERNEL_STACK_TOP

    # 保存完所有通用寄存器后，可以使用所有通用寄存器，无须使用汇编控制只使用sp寄存器，进入rust代码
    call save_context
