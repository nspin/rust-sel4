SECTIONS {
    .tdata : {}
    .tbss : {}
    .tls ALIGN(MAX(ALIGNOF(.tdata), ALIGNOF(.tbss))) (NOLOAD) : {
        __tls_start = .;
        . += SIZEOF(.tdata);
        . = ALIGN(ALIGNOF(.tbss));
        . += SIZEOF(.tbss);
        __tls_end = .;
    }
} INSERT AFTER .bss;
