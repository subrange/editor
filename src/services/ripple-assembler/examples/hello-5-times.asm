; Print "Hello World" 5 times
; Simple example for Ripple VM

.data
hello_str: .asciiz "Hello World\n"

.code
start:
    ; Initialize counter to 5
    LI R3, 5
    
loop:
    ; Check if counter is 0
    BEQ R3, R0, done
    
    ; Print Hello World
    LI R4, hello_str
    JAL R0, R0, print_string
    
    ; Decrement counter
    ADDI R3, R3, 65535  ; -1 in 16-bit
    
    ; Loop back
    JAL R0, R0, loop
    
done:
    HALT

; Simple print string subroutine
print_string:
    ; R4 contains string address
    ADD R5, RA, R0  ; Save return address
    
print_loop:
    LOAD R6, R4, 0
    BEQ R6, R0, print_done
    
    ; Output character (I/O at address 0)
    STORE R6, R0, 0
    
    ADDI R4, R4, 1
    JAL R0, R0, print_loop
    
print_done:
    JALR R0, R0, R5  ; Return