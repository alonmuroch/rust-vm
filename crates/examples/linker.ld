SECTIONS
{
    /* Dedicated section for result object at a fixed address */
    .result : {
        . = 0x100;  /* Fixed address for result object */
        . = ALIGN(8);  /* Ensure 8-byte alignment */
        *(.result*)
    }

    . = 0x400;  /* Start address for program code */

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
