
.PHONY: all
all:
	@cd lib/preflight && cargo build --release
	ls lib/preflight/target/release/
	rm -f main
	go build main.go
	ls -lh main
	@./main



