.altmacro
.macro SAVE_REG n
    sd x\n, \n*8(sp)
.endm

    .globl __save
__save:
    # 在rust代码中无法保证换栈指令位于分配指令之前，有可能先分配后换栈，这样会导致上下文分配到用户栈上；
    # 所以用汇编精细控制先换栈再分配上下文

    # 换栈，sp指向内核栈，sp原先存放的用户栈栈顶地址存放在sscratch
    csrw sscratch, sp
    ld sp, CONTEXT_START_ADDR

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

    ld sp, KERNEL_STACK_TOP
    call save_context
