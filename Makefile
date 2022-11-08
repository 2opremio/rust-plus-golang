
# use msvc to compile cgo in Windows
ifeq ($(OS),Windows_NT)
CC=cl.exe
endif

.PHONY: all
all:
	cd lib/preflight && cargo build --release
	ls lib/preflight/target/release/
	rm -f main
	CGO_LDFLAGS_ALLOW='.*' go build main.go
	ls -lh main
	./main



