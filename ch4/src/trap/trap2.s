.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .globl __trap_entry
    .globl __restore
    .globl __trap_end
    .align 2
__trap_entry:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sepc
    sd t0, 32*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t1, sscratch
    sd t1, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 33*8(sp)
    # move to kernel_sp
    li sp, 0xfffffffffffff000
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    call trap_handler

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sepc
    ld t0, 32*8(sp)
    csrw sepc, t0
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret
__trap_end:
