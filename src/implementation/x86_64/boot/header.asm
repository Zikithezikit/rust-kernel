section .multiboot_header
header_start:
    ; Magic number multi boot 2 will look for
    dd 0xe85250d6
    ; architecture
    dd 0 ; Protected mode i386
    ; Header length
    dd header_end - header_start
    ; checksum
    dd 0x100000000 - (0xe85250d6 + + 0 + (header_end - header_start))

    ; end tag
    dw 0
    dw 0
    dd 8

header_end:
