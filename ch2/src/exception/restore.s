.altmacro
.macro RESTORE_REG n
    ld x\n, \n*8(sp)
.endm

    .globl __restore
__restore:
    ld sp, CONTEXT_START_ADDR

    # 从上下文恢复除sp(x2)外的所有通用寄存器
    ld x1, 1*8(sp)
    .set n, 3
    .rept 29
        RESTORE_REG %n
        .set n, n+1
    .endr

    ld sp, 2*8(sp)

    # 返回sepc指向的地址继续执行
    sret
