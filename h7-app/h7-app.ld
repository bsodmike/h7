MEMORY
{
    SRAM (rxw) : ORIGIN = 0x24000000, LENGTH = 512K
}

EXTERN(ENTRY_POINT);

SECTIONS
{
    .entry ORIGIN(SRAM) :
    {
        KEEP(*(.entry_point))
    } > SRAM

    .text : ALIGN(8)
    {
        *    (.text .text.*)
        *    (.init)
        *    (.fini)
    } > SRAM

    .rodata : ALIGN(8)
    {
        *(.rodata .rodata.*);
        . = ALIGN(8);
    } > SRAM

    .data : ALIGN(8)
    {
        *    (.data .data.*)
        . = ALIGN(8);
    }  > SRAM

    .bss (NOLOAD) : ALIGN(8)
    {
        *    (.bss .bss.*)
        . = ALIGN(8);
    } > SRAM

    /* ## .got */
    /* Dynamic relocations are unsupported. This section is only used to detect relocatable code in
     the input files and raise an error if relocatable code is found */
    .got (NOLOAD) :
    {
        KEEP(*(.got .got.*));
    }

    /* ## Discarded sections */
    /DISCARD/ :
    {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx.*);
    }
}
