CFLAGS = $(shell pkg-config --cflags json-c)
LDFLAGS = $(shell pkg-config --libs json-c)

test: main.c json.c
	gcc -o $@ -I../../include $(CFLAGS) $^ $(LDFLAGS)

json.c: ../grammars/json.chm
	cargo run -- translate --entrypoint json::VALUE --output $@ $^
