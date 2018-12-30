INSTALL_DIR = ~/bin
EXE = swinst

build:
	cargo build --release

install:
	cp ./target/release/${EXE} ${INSTALL_DIR}/.

test:
	cargo test

check:
	cargo check

all: build install