#include <stdlib.h>
#include <string.h>

#include "../include/mailbox.h"

#ifdef __MAILBOX_DEBUG
#include <stdio.h>
size_t __malloc_count = 0;
size_t __free_count = 0;
void __mailbox_print_debug()
{
    printf("__MAILBOX_DEBUG __malloc_count = %ld\n", __malloc_count);
    printf("__MAILBOX_DEBUG __free_count = %ld\n", __free_count);
    return;
}
#endif

void *__mailbox_malloc(size_t size)
{
#ifdef __MAILBOX_DEBUG
    __malloc_count++;
#endif
    return malloc(size);
}
void __mailbox_free(void *ptr)
{
#ifdef __MAILBOX_DEBUG
    __free_count++;
#endif
    free(ptr);
}

Message *message_new(void *data, size_t size)
{
    Message *m = __mailbox_malloc(sizeof(Message));
    NULL_CHECK(m);
    m->data = data;
    m->size = size;
    m->next = NULL;
    return m;
}

Message *message_new_copy(const void *data, size_t size)
{
    NULL_CHECK(data);

    Message *m = __mailbox_malloc(sizeof(Message));
    NULL_CHECK(m);

    void *new_data = __mailbox_malloc(size);
    NULL_CHECK(new_data);
    memcpy(new_data, data, size);

    m->data = new_data;
    m->size = size;
    m->next = NULL;
    return m;
}

void message_delete(Message *msg)
{
    __mailbox_free(msg->data);
    __mailbox_free(msg);
}

Mailbox mailbox_new()
{
    Mailbox m;
    m.head = NULL;
    m.tail = NULL;
    return m;
}

bool mailbox_is_empty(Mailbox *mbox)
{
    return mbox->head == NULL;
}

void mailbox_push(Mailbox *mbox, Message *msg)
{
    if (mailbox_is_empty(mbox))
    {
        mbox->head = msg;
        mbox->tail = msg;
    }
    else
    {
        mbox->tail->next = msg;
        mbox->tail = msg;
    }
}

Message *mailbox_pop(Mailbox *mbox)
{
    if (!mailbox_is_empty(mbox))
    {
        Message *m = mbox->head;
        mbox->head = mbox->head->next;
        m->next = NULL;
        return m;
    }
    else
    {
        return NULL;
    }
}
