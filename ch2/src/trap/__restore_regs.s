# 允许宏
.altmacro

.macro RESTORE_REG n
    ld x\n, \n*8(sp)
.endm


__restore_regs:
    mv sp, a0

    # 从上下文恢复除sp(x2)外的所有通用寄存器
    ld x1, 1*8(sp)
    .set n, 3
    .rept 29
        RESTORE_REG %n
        .set n, n+1
    .endr

    # 在内核栈上释放上下文
    addi sp, sp, 34*8
    # 换栈，sp指向用户栈，sp原先存放的内核栈栈顶地址存放在sscratch
    csrrw sp, sscratch, sp

    # 返回sepc指向的地址继续执行
    sret
