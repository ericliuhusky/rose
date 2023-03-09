.altmacro
.macro RESTORE_REG n
    ld x\n, \n*8(sp)
.endm

    .section .text.trampoline
    .globl __restore
__restore:
    # 切换到用户地址空间
    csrw satp, a0
    sfence.vma

    # a0指向用户地址空间中的TrapContext地址
    ld sp, CONTEXT_START_ADDR

    # 恢复控制和状态寄存器
    ld t0, 32*8(sp)
    csrw sepc, t0

    # 从上下文恢复除sp(x2)外的所有通用寄存器
    ld x1, 1*8(sp)
    .set n, 3
    .rept 29
        RESTORE_REG %n
        .set n, n+1
    .endr

    # sp指向用户栈
    ld sp, 2*8(sp)
    
    # 返回sepc指向的地址继续执行
    sret
