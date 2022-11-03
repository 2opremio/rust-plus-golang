
.PHONY: all
all:
	@cd lib/preflight && cargo build --release
	@cp lib/preflight/target/release/libpreflight.a lib/
	rm -f main
	go build main.go
	@./main



