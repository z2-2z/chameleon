all: bench generator.so

generator.so: json.c
	gcc -g -Wall -Wextra -Wpedantic -Werror -O3 -flto -march=native -fno-stack-protector -fomit-frame-pointer -fvisibility=hidden -DCHAMELEON_VISIBLE -fPIC -shared -nostdlib -o $@ $^

bench: bench.c json.c
	gcc -Wall -Wextra -Wpedantic -Werror -I../../include -O3 -flto -march=native -fno-stack-protector -fomit-frame-pointer -g -o $@ $^

json.c: ../grammars/json.chm
	cargo run -- translate --entrypoint json::VALUE --baby -v --output $@ $^

.PHONY: clean

clean:
	@rm -fv json.c bench generator.so
