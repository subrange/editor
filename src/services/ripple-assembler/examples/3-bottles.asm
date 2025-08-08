; Simple 3 Bottles of Beer
; Counts down from 3 to 1

.data
bottles_msg: .asciiz " bottles of beer on the wall\n"
bottle_msg:  .asciiz " bottle of beer on the wall\n"
take_msg:    .asciiz "Take one down, pass it around\n"
no_more:     .asciiz "No more bottles!\n"

.code
start:
    ; Start with 3 bottles
    LI R3, 3
    
countdown:
    ; Check if we're done
    BEQ R3, R0, finished
    
    ; Print the number
    JAL R0, R0, print_digit
    
    ; Print appropriate message
    LI R4, 1
    BEQ R3, R4, one_bottle
    
    ; Multiple bottles
    LI R4, bottles_msg
    JAL R0, R0, print_string
    JAL R0, R0, after_print
    
one_bottle:
    LI R4, bottle_msg
    JAL R0, R0, print_string
    
after_print:
    ; Print take message
    LI R4, take_msg
    JAL R0, R0, print_string
    
    ; Decrement counter
    ADDI R3, R3, 65535  ; -1
    
    ; Continue loop
    JAL R0, R0, countdown
    
finished:
    LI R4, no_more
    JAL R0, R0, print_string
    HALT

; Print single digit (1-9)
print_digit:
    ADD R5, RA, R0  ; Save return address
    
    ; Convert to ASCII
    ADDI R6, R3, 48  ; Add '0'
    
    ; Output character
    STORE R6, R0, 0
    
    JALR R0, R0, R5

; Print string subroutine
print_string:
    ADD R5, RA, R0  ; Save return address
    
ps_loop:
    LOAD R6, R4, 0
    BEQ R6, R0, ps_done
    
    STORE R6, R0, 0
    
    ADDI R4, R4, 1
    JAL R0, R0, ps_loop
    
ps_done:
    JALR R0, R0, R5