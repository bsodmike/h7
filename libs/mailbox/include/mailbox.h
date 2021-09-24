#ifndef __H7_MAILBOX_H
#define __H7_MAILBOX_H

#include <stdlib.h>
#include <inttypes.h>

#ifndef bool
#define bool uint8_t
#define true 1
#define false 0
#endif

#ifdef __MAILBOX_DEBUG
void __mailbox_print_debug();
#endif

#define NULL_CHECK(expr) \
    if ((expr) == NULL)  \
    {                    \
        return NULL;     \
    }

typedef struct _Message
{
    struct _Message *next;
    void *data;
    size_t size;
} Message;

typedef struct
{
    Message *head;
    Message *tail;
} Mailbox;

Message *message_new(void *data, size_t size);
Message *message_new_copy(const void *data, size_t size);
void message_delete(Message *msg);

Mailbox mailbox_new();
bool mailbox_is_empty(Mailbox *mbox);
void mailbox_push(Mailbox *mbox, Message *msg);
Message *mailbox_pop(Mailbox *mbox);

#endif // __H7_MAILBOX_H
