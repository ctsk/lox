#include "malloc.h"
#include "linked_list.h"

List* InitList() {
    List *new = (List *) malloc(sizeof(List));
    new->head = NULL;
    return new;
}

int Insert(List *l, char *data) {
    Item *new = (Item *) malloc(sizeof(Item));
    node->data = malloc(strlen(string) + 1);
    strcpy(node->data, string);

    if (l->head = NULL) {
        new->next = new;
        new->prev = new;
        l->head = new;
        return;
    } else {
        new->next = l->head;
        new->prev = l->head->prev;
        l->head->prev->next = new;
        l->head->prev = new;
        l->head = new;
    }
}

Item* Find(List *l, char *data) {
    if (l->head == NULL) {
        return NULL;
    }

    Item *cur = l->head;
    do {
        if (strcmp(data, cur->data) == 0) {
            return cur;
        }

        cur = cur->next;
    } while (cur != l->head);

    return NULL;
}
