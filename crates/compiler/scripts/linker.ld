SECTIONS
{
    . = 0x0100;  /* Start address for first section */

    .text.entrypoint : {
        KEEP(*(.text.entrypoint))
    }

    /* Manually skip ahead by 0x40 bytes */
    . = ALIGN(4);
    . = . + 0x40;

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        *(.bss*)
    }

    /DISCARD/ : {
        *(.comment)
        *(.note.GNU-stack)
        *(.riscv.attributes)
    }
}
