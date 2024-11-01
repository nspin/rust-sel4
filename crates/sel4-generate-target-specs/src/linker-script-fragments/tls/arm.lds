SECTIONS {
    .tdata : {}
    .tbss : {}
    .tls ALIGN(MAX(8, MAX(ALIGNOF(.tdata), ALIGNOF(.tbss)))) (NOLOAD) : {
        __tls_start = .;
        . += 8;
        . = ALIGN(MAX(ALIGNOF(.tdata), ALIGNOF(.tbss)));
        . += SIZEOF(.tdata);
        . = ALIGN(ALIGNOF(.tbss));
        . += SIZEOF(.tbss);
        __tls_end = .;
    }
} INSERT AFTER .bss;
