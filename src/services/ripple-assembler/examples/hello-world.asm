; Hello World Example for Ripple VM (Fixed)

.data
hello_msg:  .asciiz "Hello, Ripple!\n"

.code
start:
    LI R3, 0
    LI R5, 2

print_loop:
    LOAD  R3, R5, 0
    BNE   R3, R0, 2
    HALT
    ADDI  R5, R5, 1  ; inc
    STORE R3, R0, 0  ; print character (I/O at address 0)
    JAL  R0, R0, print_loop
