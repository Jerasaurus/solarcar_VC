MEMORY
{
    /* Application starts at 0x08010000 (sector 4, after 48KB bootloader +
16KB config) */
    /* STM32F429VI has 2MB flash total, minus 64KB = 1984KB for app */
    FLASH : ORIGIN = 0x08010000, LENGTH = 1984K
    /* STM32F429VI has 192KB main SRAM */
    RAM : ORIGIN = 0x20000000, LENGTH = 192K
}