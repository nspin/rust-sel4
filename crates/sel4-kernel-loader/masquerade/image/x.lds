ENTRY(_start)

PHDRS
{
    all PT_LOAD AT (0);
}

SECTIONS
{
    . = 0;

    .text : {
        KEEP(*(.text.header))
        *(.text*)
    } :all

    .rodata : {
        *(.rodata*)
    } :all

    .data : {
        *(.data)
    } :all

    ASSERT(SIZEOF(.data) == 0, ".data section present")

    .bss : {
        *(.bss)
        *(COMMON)
    } :all

    ASSERT(SIZEOF(.bss) == 0, ".bss section present")

    . = ALIGN(8);

    _payload_start = .;

    .symtab 0 : { *(.symtab) }
    .strtab 0 : { *(.strtab) }
    .shstrtab 0 : { *(.shstrtab) }

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}
