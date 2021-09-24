#define __MAILBOX_DEBUG

#include "../src/mailbox.c"
#include <stdio.h>

int main(int argc, char *argv[], char *env[])
{
    Mailbox mbox = mailbox_new();

    mailbox_push(&mbox, message_new_copy("one", 4));
    mailbox_push(&mbox, message_new_copy("two", 4));
    mailbox_push(&mbox, message_new_copy("three", 6));
    mailbox_push(&mbox, message_new_copy("four", 5));
    mailbox_push(&mbox, message_new_copy("five", 5));

    for (size_t i = 0; i < 10; i++)
    {
        char msg[] = "msg 0";
        msg[4] = msg[4] + i;
        mailbox_push(&mbox, message_new_copy(msg, strlen(msg) + 1));
    }

    while (!mailbox_is_empty(&mbox))
    {
        Message *m = mailbox_pop(&mbox);
        if (m == NULL)
        {
            break;
        }
        printf("size: %ld, data: %s, next: %p\n", m->size, (char *)m->data, m->next);
        message_delete(m);
    }

#ifdef __MAILBOX_DEBUG
    __mailbox_print_debug();
#endif
}
