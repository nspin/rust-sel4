PHDRS
{
    all PT_LOAD AT (0);
}

SECTIONS
{
    . = 0;

    .text :
    {
        _text = .;
        *(.text.start)
        *(.text*)
        *(.rodata)
        _text_end = .;
    } :all

    .data :
    {
        _data = .;
        *(.data)
        _data_end = .;
    } :all

    .bss :
    {
        _bss = .;
        *(.bss)
        *(COMMON)
        . = ALIGN(4);
        _bss_end = .;
    } :all
}
