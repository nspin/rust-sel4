ENTRY(_start)

PHDRS
{
    all PT_LOAD AT (0);
}

SECTIONS
{
    /*
    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame.*)
        *(.eh_frame_hdr)
        *(.gcc_except_table*)
        *(.note.GNU-stack)
        *(.comment)
    }
    */

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

    . = ALIGN(8);

    _payload_start = .;
}
