BIN_NAME=coretype
SRC = $(wildcard src/*.rs)

build : $(SRC)
	cargo build --release


install_local : build
	cp ./target/release/${BIN_NAME} ~/.local/bin/

install : build
	cp ./target/release/${BIN_NAME} /usr/local/bin/

help:
	@echo $(SRC)