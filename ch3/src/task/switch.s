# 允许宏
.altmacro

.macro SAVE_REG n
    sd s\n, (\n+2)*8(a0)
.endm

.macro RESTORE_REG n
    ld s\n, (\n+2)*8(a1)
.endm


    .section .text
    .globl __switch
__switch:
    # 在当前任务上下文中保存返回地址即调用__switch函数的地址，
    # 保存栈指针和被调用者保存寄存器s0~s11
    sd ra, 0(a0)
    sd sp, 8(a0)
    .set n, 0
    .rept 12
        SAVE_REG %n
        .set n, n + 1
    .endr

    # 从下个任务上下文中恢复返回地址
    # 恢复栈指针和被调用者保存寄存器s0~s11
    ld ra, 0(a1)
    ld sp, 8(a1)
    .set n, 0
    .rept 12
        RESTORE_REG %n
        .set n, n + 1
    .endr

    # 跳转到下个任务上下文保存的返回地址继续执行
    ret
