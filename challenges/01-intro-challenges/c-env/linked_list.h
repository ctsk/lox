typedef struct _Item {
    char *data;
    struct _Item *prev;
    struct _Item *next;
} Item;

typedef struct List {
   Item *head;
} List;

List* InitList();
void Insert(List *l, char *data);
Item* Find(List *l, char *data);
