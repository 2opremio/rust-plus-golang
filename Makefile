
.PHONY: all
all:
	@cd lib/preflight && cargo build --release
	rm -f main
	go build main.go
	@./main



