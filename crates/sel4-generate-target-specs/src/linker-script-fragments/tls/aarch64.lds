SECTIONS {
    .tls ALIGN(MAX(16, MAX(ALIGNOF(.tdata), ALIGNOF(.tbss)))) (NOLOAD) : {
        __tls_start = .;
        . += 16;
        . = ALIGN(MAX(ALIGNOF(.tdata), ALIGNOF(.tbss)));
        . += SIZEOF(.tdata);
        . = ALIGN(ALIGNOF(.tbss));
        . += SIZEOF(.tbss);
        __tls_end = .;
    }
} INSERT AFTER .bss;
