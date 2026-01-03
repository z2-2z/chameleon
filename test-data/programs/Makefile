
main: main.c json.c
	gcc -Wall -Wextra -Wpedantic -Werror -I../../include -fsanitize=address -O0 -g -o $@ $^

json.c: ../grammars/json.chm
	cargo run -- translate --entrypoint json::VALUE --output $@ $^