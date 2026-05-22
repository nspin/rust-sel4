ENTRY(_start)

PHDRS
{
    all PT_LOAD AT (0);
}

SECTIONS
{
    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame.*)
        *(.eh_frame_hdr)
        *(.gcc_except_table*)
        *(.note.GNU-stack)
        *(.comment)
    }

    . = 0;

    .text :
    {
        KEEP(*(.text.header))
        *(.text*)
    } :all

    .rodata :
    {
        *(.rodata*)
    } :all

    /*
    ASSERT(SIZEOF(.rodata) == 0, ".rodata section present")
    */

    .data :
    {
        *(.data)
    } :all

    ASSERT(SIZEOF(.data) == 0, ".data section present")

    .bss :
    {
        *(.bss)
        *(COMMON)
    } :all

    ASSERT(SIZEOF(.bss) == 0, ".bss section present")

    .got : { *(.got .got.*) }
    .got.plt : { *(.got.plt) }
    .rela.dyn : { *(.rela.dyn .rela.*) }
    .data.rel.ro : { *(.data.rel.ro .data.rel.ro.*) }

    ASSERT(SIZEOF(.got) == 0, ".got present")
    ASSERT(SIZEOF(.got.plt) == 0, ".got.plt present")
    ASSERT(SIZEOF(.rela.dyn) == 0, ".rela.dyn present")
    ASSERT(SIZEOF(.data.rel.ro) == 0, ".data.rel.ro present")

    . = ALIGN(16);

    _payload_start = .;
}
