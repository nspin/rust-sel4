SECTIONS {
    .tls ALIGN(MAX(ALIGNOF(.tdata), ALIGNOF(.tbss))) (NOLOAD) : {
        __tls_start = .;
        . += SIZEOF(.tdata);
        . = ALIGN(ALIGNOF(.tbss));
        . += SIZEOF(.tbss);
        . = ALIGN(MAX(ALIGNOF(.tdata), ALIGNOF(.tbss)));
        . += 16;
        __tls_end = .;
    }
} INSERT AFTER .bss;
