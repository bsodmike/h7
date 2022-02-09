MEMORY
{
    SRAM (rxw) : ORIGIN = 0x24000000, LENGTH = 512K
    /*SRAM (rxw) : ORIGIN = 0x30000000, LENGTH = 128K*/
    /*SRAM (rxw) : ORIGIN = 0x00000000, LENGTH = 64K*/
}

EXTERN(ENTRY_POINT);

SECTIONS
{
    .entry ORIGIN(SRAM) :
    {
        KEEP(*(.entry_point))
    } > SRAM

    .text : ALIGN(4)
    {   *    (.entry_point_fn)
        *    (.text .text.*)
        *    (.init)
        *    (.fini)
    } > SRAM

    .rodata : ALIGN(4)
    {
        *(.rodata .rodata.*);
        . = ALIGN(4);
    } > SRAM

    .data : ALIGN(4)
    {
        *    (.data .data.*)
        . = ALIGN(4);
    }  > SRAM

    .bss (NOLOAD) : ALIGN(4)
    {
        *    (.bss .bss.*)
        . = ALIGN(4);
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
