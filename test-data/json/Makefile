CFLAGS = $(shell pkg-config --cflags libjq)
LDFLAGS = $(shell pkg-config --libs libjq)

test: main.c json.c
	gcc -o $@ -I../../include $(CFLAGS) $^ $(LDFLAGS)

json.c: ../grammars/json.chm
	cargo run -- translate --entrypoint json::VALUE --output $@ $^
