; Hello World Example for Ripple VM (Fixed)

.data
hello_msg:  .asciiz "Hello, Ripple!\n"

.code
start:
    LI S0, 0
    LI S1, 2

print_loop:
    LOAD  S0, 0, S1
    BNE   S0, R0, 2
    HALT
    ADDI  S1, S1, 1  ; inc
    STORE S0, R0, 0  ; print character (I/O at address 0)
    JAL  R0, R0, print_loop
