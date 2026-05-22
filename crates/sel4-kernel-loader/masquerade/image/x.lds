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

    ASSERT(SIZEOF(.rodata) == 0, ".rodata section present")

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

    . = ALIGN(16);

    _payload_start = .;
}
