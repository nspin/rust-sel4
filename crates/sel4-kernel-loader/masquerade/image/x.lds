ENTRY(_start)

PHDRS
{
    all PT_LOAD AT (0);
}

SECTIONS
{
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

    .bss :
    {
        *(.bss)
        *(COMMON)
    } :all
}
