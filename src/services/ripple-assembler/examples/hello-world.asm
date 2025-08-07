; Hello World Example for Ripple VM (Fixed)
; This version matches the working handwritten code

.data
; Note: In the working version, data starts at offset 2
; We need to add padding
padding:    .space 2        ; Reserve 2 bytes at start
hello_msg:  .asciiz "Hello, Ripple!\n"

.code
start:
    ; R3 will hold the character being loaded
    LI R3, 0
    
    ; R4 holds the jump address for the loop (instruction 4)
    LI R4, 4
    
    ; R5 holds the memory pointer (starts at 2 where our string begins)
    LI R5, 2
    
    ; Jump to print_loop (this is redundant but matches the working version)
    JALR R4, R0, R4
    
print_loop:
    ; Load character from memory at address in R5
    LOAD R3, R5, 0
    
    ; If character is not zero, continue printing
    BNE R3, R0, 1
    
    ; Character was zero, halt
    HALT
    
    ; Increment memory pointer
    ADDI R5, R5, 1
    
    ; Output character to I/O (address 0)
    STORE R3, R0, 0
    
    ; Jump back to print_loop
    JALR R4, R0, R4