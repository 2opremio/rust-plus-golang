package main

// NOTE: There should be NO space between the comments and the `import "C"` line.
// The -ldl is sometimes necessary to fix linker errors about `dlsym`.

/*
#cgo LDFLAGS: ./lib/libhello.a -ldl
#include "./lib/hello.h"
*/
import "C"
import "fmt"

//export MyGoPrint
func MyGoPrint(str *C.char) {
	fmt.Println(C.GoString(str))
}

func main() {
	C.hello(C.CString("world"))
	C.whisper(C.CString("this is code from the static library"))
}
