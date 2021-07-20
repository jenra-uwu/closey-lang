# Closey
Closey is a functional programming language designed to have as few features as possible but still be easy to use and compiles to efficient machine code.

## Features
- Functions
- Function applications
- Partial function application
- Union types
- Match expressions
- Closures
- Optimisation of church numerals and cons boxes to corresponding native types

## Examples
### Identity
```ocaml
(\a: 'a . a) (\a: 'a . a)
```

## Build
Just type in the following:
```bash
git clone https://github.com/jenra-uwu/closey-lang && cd closey-lang && cargo build
```

### Building the library
```bash
cd lib && make
```

Note: Currently the focus is on Linux and macOS support, Linux being the more stable of the two. If you want it to run on a Windows computer, either a) install WSL, or b) get a better operating system.

## Building a program
```bash
closeyc build -o file.o -- file.closey
ld -o file file.o path/to/libclosey.a
./file
```

## Progress
See TODO.md. Everything is highly experimental. Be cautious: code may be explosive.
