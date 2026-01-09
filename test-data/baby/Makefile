bench: bench.c json.c
	gcc -Wall -Wextra -Wpedantic -Werror -I../../include -O3 -flto -march=native -fno-stack-protector -fomit-frame-pointer -g -o $@ $^

json.c: ../grammars/json.chm
	cargo run -- translate --entrypoint json::VALUE --baby --output $@ $^

.PHONY: clean

clean:
	@rm -fv json.c	bench
