.altmacro

.macro SAVE_REG n
    sd x\n, \n*8(sp)
.endm

.macro RESTORE_REG n
    ld x\n, \n*8(sp)
.endm


__save_regs:
    # 在rust代码中无法保证换栈指令位于分配指令之前，有可能先分配后换栈，这样会导致上下文分配到用户栈上；
    # 所以用汇编精细控制先换栈再分配上下文

    # 换栈，sp指向内核栈，sp原先存放的用户栈栈顶地址存放在sscratch
    csrrw sp, sscratch, sp
    # 在内核栈上分配上下文
    addi sp, sp, -34*8

    # 此时只有sp寄存器可以使用，用户栈栈顶地址已经保存在sscratch，即使改变sp寄存器也可从sscratch恢复
    # 此时使用其它寄存器，会导致其它寄存器的值被改变覆盖原有值，使得其它寄存器无法恢复
    # 在rust代码中无法保证使用哪个寄存器，所以用汇编精细控制只使用sp寄存器

    # 在上下文中保存除sp(x2)外的所有通用寄存器
    sd x1, 1*8(sp)
    .set n, 3
    .rept 29
        SAVE_REG %n
        .set n, n+1
    .endr

    # 在保存完所有通用寄存器后，就可以自由使用所有通用寄存器

    # 保存控制和状态寄存器    
    csrr t0, sstatus
    csrr t1, sepc
    csrr t2, sscratch
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 保存用户栈
    sd t2, 2*8(sp)

    mv a0, sp
    call trap_handler


__restore_regs:
    mv sp, a0

    # 恢复控制和状态寄存器
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # 恢复用户栈
    csrw sscratch, t2

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
