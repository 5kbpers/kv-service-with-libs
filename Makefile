all: build

build:
	./generate_proto.sh
	cargo build 

run:
	./generate_proto.sh
	cargo run --bin kv-server

.PHONY: all
