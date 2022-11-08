
ifndef CARGO_BUILD_TARGET
export CARGO_BUILD_TARGET=$(shell rustc -vV | sed -n 's|host: ||p')
endif

ifeq ($(OS),Windows_NT)
# This assumes that the Rust compiler should be using a -gnu target (i.e. MinGW compiler) in Windows
# (I (fons) am not even sure if CGo supports MSVC, see https://github.com/golang/go/issues/20982 )
export CGO_LDFLAGS=./lib/preflight/target/${CARGO_BUILD_TARGET}/release/libpreflight.a -ldl -lm -lws2_32 -lbcrypt -luserenv
else
export CGO_LDFLAGS=./lib/preflight/target/${CARGO_BUILD_TARGET}/release/libpreflight.a -ldl -lm
endif

.PHONY: all
all: main
	./main

lib/preflight/target/${CARGO_BUILD_TARGET}/release/libpreflight.a: lib/preflight.h lib/preflight/Cargo.toml lib/preflight/src/lib.rs
	cd lib/preflight && cargo build --release

main: ./lib/preflight/target/${CARGO_BUILD_TARGET}/release/libpreflight.a lib/preflight.h
	go build main.go

.PHONY: clean
clean:
	rm -r main lib/preflight/target






