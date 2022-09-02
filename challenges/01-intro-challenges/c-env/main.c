#include <stdio.h>
#include "linked_list.h"
#include <assert.h>

int main()
{
    List *l = InitList();

    Insert(l, "one");
    Insert(l, "two");
    Insert(l, "three");

    int f = Find(l, "two");
    assert(f > 0);

    int f = Find(l, "four");
    assert(f == 0);
}
