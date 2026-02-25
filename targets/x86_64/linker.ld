ENTRY(start)

PHDRS
{
    boot PT_LOAD;
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
    bss PT_LOAD;
}

SECTIONS
{
    . = 1M;

    .boot : { KEEP(*(.multiboot_header)) } :boot
    .text : { *(.text) } :text
    .rodata : { *(.rodata) } :rodata
    .data : { *(.data) } :data
    .bss : { *(.bss) } :bss
}
