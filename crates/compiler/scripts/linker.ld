SECTIONS
{
    . = 0x10000;

    .text : {
        KEEP(*(.text.keep))
        *(.text*)
    }

    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    .bss : { *(.bss*) }
}
