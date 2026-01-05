
generator.so: json.c
	gcc -Wall -Wextra -Wpedantic -Werror -O3 -flto -march=native -fno-stack-protector -fomit-frame-pointer -fvisibility=hidden -DCHAMELEON_VISIBLE -DOMIT_CHAMELEON_MUTATE -DOMIT_CHAMELEON_INIT -DOMIT_CHAMELEON_DESTROY -fPIC -shared -o $@ $^

json.c: ../grammars/json.chm
	cargo run -- translate --entrypoint json::VALUE --output $@ $^

.PHONY: clean

clean:
	@rm -fv json.c generator.so
