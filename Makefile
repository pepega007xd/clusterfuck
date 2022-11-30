CFLAGS=-std=c99 -Wall -Wextra -Werror
LDFLAGS=-lm -ldl
.DEFAULT_GOAL := cluster

cluster: cripple
	gcc $(CFLAGS) cluster.c $(LDFLAGS) -o cluster

cripple: polotovar.c polotovar.rs
	bash cripple.sh

clean:
	rm -f cluster.c fuck.rs fuck.so cluster
