
ifndef CARGO_BUILD_TARGET

endif

export CGO_LDFLAGS=./lib/preflight/target/${CARGO_BUILD_TARGET}/release/libpreflight.a -ldl -lm


.PHONY: all
all:
	cd lib/preflight && cargo build --release
	ls lib/preflight/target/${CARGO_BUILD_TARGET}/release/
	rm -f main
	go build main.go
	ls -lh main
	./main



