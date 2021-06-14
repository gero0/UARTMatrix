#target extended-remote :3333

# print demangled symbols
set print asm-demangle on

load

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

monitor arm semihosting enable

# start the process but immediately halt the processor
#stepi