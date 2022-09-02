run: main
	@ ./main

main: main.c linked_list.o
	@ gcc $^ -o main

linked_list.o: linked_list.c
	@ gcc -c linked_list.c


