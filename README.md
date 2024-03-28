# binc
![image info](https://img.shields.io/badge/status-not%20ready-red)
![image info](https://img.shields.io/badge/cargo-1.54.0&ndash;nightly-blue)  
binc is a BINary Calculator with REPL user interface.

It prints *the number* in binary format and reads commands from standard input
to be executed upon *the number* or particular bits of *the number*.

Or it reads commands from command line argument and prints the result.

There is binc's output with 42 assigned to *the number*.
```text
$ binc                                           # shell's command line
                                                 #
        0x2a         0d42          0o52          # number representation in different radixes
   31     24  23     16  15      8  7       0    # bit indexes
+  0000 0000  0000 0000  0000 0000  0010 1010    # the number in binary radix
  0  28 27      20 19      12 11       4 3       # bit indexes
(binc)                                           # binc's prompt
```

"bit indexes" intended to help user to understand what index each bit has. "*the number* in binary radix" is split up into a bytes and half-bytes sequences of 1 and 0.

On the line called "*the number* in binary radix" there is a '+' on the left, it represents that the number is positive, `-` represents a negative number.

The '  0' under the '+' means carry, it is a state of the "carry bit".
Carry bit indicates whether the overflow occurred after operation or not.

To run binc in non-interactive mode run it with key `-e` and pass a list of binc's commands separated by `;`, see [examples](#Examples).

## Usage
Command syntax is:  
 - X operator Y
 - operator X
 - command

, where *X* is lvalue, a **range** only; *Y* is rvalue operand, a number or **range**; *operator* is some math operation, like `>>` or `+`; *command* is literal command.
Spaces are ignored like in Fortran ☺️  
**number** is a numeric literal with prefixes 0x, 0d, 0b, 0h(0x) to specify corresponding radix. Can be prefixed with minus to make it negative.  
First operand *X* may be omitted, by default it is range `[]` - *the whole number*, `[i:j]` - bits from `j` to `i`, j <= i, range `[i]` operates on exactly i-th bit. Half-range `[i:]` means from i-th bit inclusive to the lowest bit, `[:i]` means from the highest bit to the i-th bit inclusive. The *Y* operand also can be a character written in single quotes, like `'x'`, unicode is supported, but emoji aren't. To input `'` type `'''`.
If *operator* is omitted than binc do assignment `=` operation by default. To omit an operator, *X* also must be omitted. Only *Y* operand is necessary. If one typed just 123 without specifying an operator, the number 123 will be set.  
Empty command line repeats last command.

binc knows `help` command with no arguments, which will print all possible commands, operators and syntax tips.

To quit binc press `[CTRL+C]`, `[CTRL+D]` or `[CTRL+Q]`.

## Binary Operators
| operator | description | operator | description                              |
|----------|:------------|:---------|------------------------------------------|
| `=`      | assignment  | `==`     | comparison, only prints result           |
| `+`      | add         | `>>`     | signed shift right, 1000 >> 1 is 1100    |
| `-`      | subtract    | `>>>`    | unsigned shift right (1000 >>> 1 == 0100 |
| `*`      | multiply    | `<<`     | shift left                               |
| `/`      | divide      | `>`      | greater, only prints result              |
| `%`      | remainder   | `<`      | less, only prints result                 |
| `^`      | bitwise xor | `~>>`    | signed cyclic shift right                |
| `&`      | bitwise and | `<<~`    | cyclic shift left                        |
| &#124;   | bitwise or  | `pow`    | exponentiation                           |
| `<>`     | exchange    | `root`   | square root                              |

## Unary operators
| operator | description  | operator | description         |
|----------|:-------------|:---------|---------------------|
| `~`      | bitwise not  | `!`      | arithmetic negation |
| `shf`    | shuffle bits | `rnd`    | randomize           |
| `rev`    | reverse      |          |                     |

## Commands
| command    | action                                                            |
|------------|:------------------------------------------------------------------|
| `help`     | prints all operators, commands and syntactic tips                 |
| `undo`     | undo last operation                                               |
| `reduo`    | redo operation, that was "undo"ed                                 |
| `intX`     | treat *the number* as an integer, X - bits: 8, 16, 32, 64, 128.   |
| `floatX`   | treat *the number* as a floating point one, X - bits. (not ready) |
| `fixedX`   | treat *the number* as a fixed point one, X - bits. (not ready)    |
| `printf`   | prints *the number* in a specified format. (not ready)            |
| `signed`   | treat *the number* as singed int (bit width does not change).     |
| `unsigned` | treat *the number* as unsigned int (bit width does not change) .  |

## Examples
`(binc) 42` sets *the number* to 42.  
`(binc) +1` add 1 to *the number*.  
`(binc) -1` set all bits to 1 if number was 0. Actually subtracts -1.  
`(binc) &0` bitwise conjunction of bits of *the number* and 0, set all bits to 0.  
`(binc) /2;>>1` divides *the number* by 2 and shifts right one bit.  
`(binc) [31]=1` set 31-st bit of *the number* to 1, the number becomes negative.    
`(binc) [31:24] cnt 0` count zero bits in range from 24-th bit to 31-st bit inclusive.  
`(binc) [15:4] printf x` prints bits from 4 to 15 inclusive as hexadecimal (not ready).  
`(binc) [15:0] <> [31:16]` swap values of lower and higher bits of the number.  
`(binc) '愛'` set a kanji code to *the number*.  
`(binc) !` do arithmetic negation on *the number*. X becomes -X.  
`$ binc -e '-1;[7:0]&0'` sets negative number (all bits become 1), than nulls first byte, prints.  
`$ binc -e '123' -fx` initialize with 123, print as hexadecimal.
`$ binc -e '123' -p` initialize with 123, print as binary with 32 digits, padded from left with zeroes.

## Build and Install
[Install](https://doc.rust-lang.org/cargo/getting-started/installation.html) Rust, compile and run:
```shell
cargo build --release
cargo run
```

## Miscellaneous
- binc is colored
![image info](./pictures/binc-output.png)
- It is possible to make binc to print logs by passing flag -v in command line. To report a bug please do `binc -vvvv -e '...'` with commands which caused a bug, and copy output to the issues of the repo. 
- Set history size by passing --history=X in command line, where X is a desired history length.
- `$ binc -e -- '-1'` to set negative number does not work.
