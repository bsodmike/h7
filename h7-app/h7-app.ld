MEMORY
{
    FLASH (rx) : ORIGIN = 0x24000000, LENGTH = 512K
    RAM   (rw) : ORIGIN = 0x30000000, LENGTH = 288K
}

EXTERN(ENTRY_POINT);

SECTIONS
{
    .entry ORIGIN(FLASH) :
    {
        KEEP(*(.entry_point))
    } > FLASH

    .text :
    {
        *    (.text .text.*)
        *    (.init)
        *    (.fini)
    } > FLASH

    .rodata : ALIGN(4)
    {
        *(.rodata .rodata.*);
        . = ALIGN(4);
    } > FLASH

    .data : ALIGN(4)
    {
        *    (.data .data.*)
        . = ALIGN(4);
    }  > RAM

    .bss (NOLOAD) : ALIGN(4)
    {
        *    (.bss .bss.*)
        . = ALIGN(4);
    } > RAM

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
