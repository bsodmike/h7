#include "../../../h7-applib/dist/h7.h"

int32_t h7_main()
{
    h7_puts((uint8_t *)"Hello from C testapp!\n");

    // Test alloc
    // uint8_t *p = h7_malloc(1024);
    // h7_free(p);

    while (1)
    {

        uint8_t c = h7_getc();
        if (c == 'b')
        {
            break;
        }
        else if (c != 0)
        {
            h7_putc(c);
        }
    }

    return 0;
}
