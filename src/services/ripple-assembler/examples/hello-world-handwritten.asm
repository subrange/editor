; Hello World - Matching the handwritten version exactly
; This demonstrates the exact pattern from the working handwritten code

.data
; The handwritten version expects data at offset 2
padding:    .space 2
hello_msg:  .asciiz "Hello, World!\n"

.code
    ; R3 is loader (character being loaded)
    LI R3, 0
    
    ; R4 holds jump address (4) - stable throughout
    LI R4, 4
    
    ; R5 is memory pointer, starts at 2
    LI R5, 2
    
    ; Jump to load instruction (unnecessary but matches handwritten)
    JALR R4, R0, R4
    
    ; Load letter from address in R5
    LOAD R3, R5, 0
    
    ; If not zero, continue (branch offset 1 = skip halt)
    BNE R3, R0, 1
    
    ; Halt if character was zero
    HALT
    
    ; Increment memory pointer
    ADDI R5, R5, 1
    
    ; Send letter to I/O output (address 0)
    STORE R3, R0, 0
    
    ; Jump back to load instruction
    JALR R4, R0, R4