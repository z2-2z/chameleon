all: main bench

bench: bench.c json.c
	gcc -Wall -Wextra -Wpedantic -Werror -I../../include -O3 -flto -march=native -fno-stack-protector -fomit-frame-pointer -o $@ $^

main: main.c json.c
	gcc -Wall -Wextra -Wpedantic -Werror -I../../include -fsanitize=address -O0 -g -o $@ $^

json.c: ../grammars/json.chm
	cargo run -- translate --entrypoint json::VALUE --output $@ $^

.PHONY: clean

clean:
	@rm -fv json.c main	bench
