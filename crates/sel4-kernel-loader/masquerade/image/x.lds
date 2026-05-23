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

    .rela.dyn : {
        . = ALIGN(8);
        __rela_start = .;
        KEEP(*(.rela.dyn))
        __rela_end = .;
    } :all

    .data.rel.ro : {
        *(.data.rel.ro .data.rel.ro.*)
    } :all

    .got : {
        *(.got)
    } :all

    . = ALIGN(8);

    _payload_start = .;

    .symtab 0 : { *(.symtab) }
    .strtab 0 : { *(.strtab) }
    .shstrtab 0 : { *(.shstrtab) }

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.dynsym)
        *(.dynstr)
        *(.hash)
        *(.gnu.hash)
        *(.dynamic)
    }
}
