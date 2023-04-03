SECTIONS
{
    . = 0x80200000;

    skernel = .;
    .text : {
        *(.text.entry)
        strampoline = .;
        *(.text.trampoline)
        etrampoline = .;
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ekernel = .;
}
