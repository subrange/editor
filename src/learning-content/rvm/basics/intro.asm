; Introduction to Ripple VM Assembly
; RVM is a 16-bit RISC-like virtual machine

; === Data Section ===
; Define initialized data here
.data
    hello_msg: .asciiz "Hello, RVM!\n"
    numbers: .word 10, 20, 30, 40, 50
    counter: .word 0

; === Code Section ===
.code
start:
    ; === Basic Instructions ===
    
    ; Load immediate value into register
    LI R1, 100      ; R1 = 100
    LI R2, 200      ; R2 = 200
    
    ; Arithmetic operations
    ADD R3, R1, R2  ; R3 = R1 + R2 (300)
    SUB R4, R2, R1  ; R4 = R2 - R1 (100)
    
    ; Immediate arithmetic
    ADDI R5, R1, 50 ; R5 = R1 + 50 (150)
    SUBI R6, R2, 25 ; R6 = R2 - 25 (175)
    
    ; === Memory Operations ===
    
    ; Store and load
    LI R7, 1000     ; Address
    STORE R3, R7, 0 ; Store R3 at address 1000
    LOAD R8, R7, 0  ; Load from address 1000 into R8
    
    ; === Control Flow ===
    
    ; Conditional branches
    LI R9, 5
    LI R10, 5
    BEQ R9, R10, equal_label  ; Branch if R9 == R10
    
    ; This won't execute
    LI R11, 999
    
equal_label:
    LI R11, 111     ; This will execute
    
    ; === Loop Example ===
    ; Print hello message character by character
    
    LI R3, 0        ; Initialize index
    
print_loop:
    ; Load character from string
    LOAD R4, R3, hello_msg
    
    ; Check for null terminator
    BEQ R4, R0, done
    
    ; Output character (store to I/O address 0)
    STORE R4, R0, 0
    
    ; Increment index
    ADDI R3, R3, 1
    
    ; Jump back to loop start
    JAL R0, print_loop
    
done:
    ; Program complete
    HALT