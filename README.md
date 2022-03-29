# binc
![image info](https://img.shields.io/badge/status-not%20ready-red)
![image info](https://img.shields.io/badge/cargo-1.54.0&ndash;nightly-blue)  
binc is a command line interface BINary Calculator.

It prints the number in binary format and reads commands from standard input to be executed upon the number or particular bits of the number.

There is binc's output initialized with number 42.
```text
$ binc                                           | shell's command line
                                                 |
        0x2a         0d42          0o52          | number representation in different radixes
   31     24  23     16  15      8  7       0    | bit indexes
+  0000 0000  0000 0000  0000 0000  0010 1010    | the number in binary radix
c 0  28 27      20 19      12 11       4 3       | bit indexes
(binc)                                           | binc's prompt
```

"bit indexes" intended to help user to understand what index each bit has. "the number in binary radix" is split up into a bytes and half-bytes 0/1 sequences.

On the line called "the number in binary radix" there is a '+' on the left, it represents that the number is positive, '-' for negative number.

The 'c 0' under the '+' means carry, it as a state of "carry bit", whether the overflow occurred after operation or not.

## Usage
Command syntax is:  
 - X operator Y
 - operator X
 - command

, where *X* is lvalue, a **range** only; *Y* is rvalue operand, a **number** or **range**; *operator* is some math operation, like `>>` or `+`; *command* is literal command.
Spaces are ignored like in Fortran ☺️  
**number** is a numeric literal with prefixes 0x, 0d, 0b, 0h(0x) to specify corresponding radix. Can be prefixed with minus to make it negative.  
First operand *X* may be omitted, by default it is range `[]` - the whole number. If *operator* is omitted than binc do assignment operation by default. Only *Y* operand is necessary. If one typed just 123, the number 123 will be set.

binc knows `help` command with no arguments, which will print all possible commands and operators and syntax tips for using them.

## Examples

binc is colored
![image info](./pictures/binc-output.png)
