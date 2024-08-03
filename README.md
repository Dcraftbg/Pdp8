# Pdp8 project
**NOTE:** Currently only an assembler is implemented 
Pdp8 is a collection of a simple tools for interacting with the pdp8 architecture.

## Quickstart
To get started you'll need to have rust and cargo installed.

### Building
```sh
cargo build --release
```
Should generate:
```
target/release/pdp8asm
```

## Pdp8 assembler
The Pdp8 assembler is very simple, only supporting basic instructions + iot currently:
```
Integer literals: $(0b|0x|0o)[0-9,a-z,A-Z] 
Instructions: [a-z,A-Z]
Modes: I|Z|IZ
Current instruction: $                # (Only lexed, but not used currently)
```
To build a program use:
```
pdp8asm hello.pdp8 -o hello.bin
```
Which will generate hello.bin with the raw instructions.
