
ifndef CARGO_BUILD_TARGET

endif

ifeq ($(OS),Windows_NT)
export CGO_LDFLAGS=./lib/preflight/target/${CARGO_BUILD_TARGET}/release/preflight.lib -ldl -lm
else
export CGO_LDFLAGS=./lib/preflight/target/${CARGO_BUILD_TARGET}/release/libpreflight.a -ldl -lm
endif

.PHONY: all
all:
	cd lib/preflight && cargo build --release
	ls lib/preflight/target/${CARGO_BUILD_TARGET}/release/
	rm -f main
	go build main.go
	ls -lh main
	./main



