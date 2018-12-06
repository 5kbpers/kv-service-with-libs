BIN_PATH = $(CURDIR)/bin
CARGO_TARGET_DIR ?= $(CURDIR)/target

all: build

build:
	./generate_proto.sh
	cargo build 
	cp $(CARGO_TARGET_DIR)/debug/kv-client $(CARGO_TARGET_DIR)/debug/kv-server $(BIN_PATH)/

.PHONY: all
