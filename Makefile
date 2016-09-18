all:
	gcc -o server -g server.c

run: all
	./server

dbg: all
	gdb ./server
