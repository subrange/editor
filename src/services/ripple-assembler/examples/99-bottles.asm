; 99 Bottles of Beer Example for Ripple VM
; Demonstrates loops, arithmetic, and conditional branching

.data
bottle_str:     .asciiz " bottles of beer on the wall,\n"
bottle_str2:    .asciiz " bottles of beer.\n"
take_str:       .asciiz "Take one down, pass it around,\n"
one_bottle:     .asciiz " bottle of beer on the wall,\n"
one_bottle2:    .asciiz " bottle of beer.\n"
no_more:        .asciiz "No more bottles of beer on the wall!\n"
newline:        .asciiz "\n"

.code
start:
    ; Initialize counter to 99
    LI R3, 99
    
main_loop:
    ; Check if we've reached 0
    BEQ R3, R0, finish
    
    ; Print number
    JALR RA, R3, print_number
    
    ; Check if singular
    LI R4, 1
    BEQ R3, R4, print_one_bottle
    
    ; Print plural form
    LI R5, bottle_str
    JALR RA, R5, print_string
    
    ; Print number again
    JALR RA, R3, print_number
    
    LI R5, bottle_str2
    JALR RA, R5, print_string
    
    JAL after_bottle_print
    
print_one_bottle:
    ; Print singular form
    LI R5, one_bottle
    JALR RA, R5, print_string
    
    ; Print number again
    JALR RA, R3, print_number
    
    LI R5, one_bottle2
    JALR RA, R5, print_string
    
after_bottle_print:
    ; Print "Take one down..."
    LI R5, take_str
    JALR RA, R5, print_string
    
    ; Decrement counter
    SUBI R3, R3, 1
    
    ; Print newline
    LI R5, newline
    JALR RA, R5, print_string
    
    ; Continue loop
    JAL main_loop
    
finish:
    ; Print final message
    LI R5, no_more
    JALR RA, R5, print_string
    HALT

; Subroutine: print_string
; Input: R5 = string address
; Uses: R6, R7
print_string:
    ; Save return address
    ADD R12, RA, R0
    
ps_loop:
    LOAD R6, R5, 0
    BEQ R6, R0, ps_done
    
    ; Output character
    LI R7, 0xFFFF
    STORE R6, R7, 0
    
    ADDI R5, R5, 1
    JAL ps_loop
    
ps_done:
    ; Restore return address and return
    JALR R0, R12, 0

; Subroutine: print_number
; Input: R3 = number to print (0-99)
; Uses: R8, R9, R10, R11
print_number:
    ; Save return address
    ADD R13, RA, R0
    
    ; Check if >= 10
    LI R8, 10
    SLT R9, R3, R8
    BNE R9, R0, print_single_digit
    
    ; Two digits - divide by 10
    ; Simple division by repeated subtraction
    ADD R10, R0, R0  ; quotient
    ADD R11, R3, R0  ; remainder
    
div_loop:
    SLT R9, R11, R8
    BNE R9, R0, div_done
    SUB R11, R11, R8
    ADDI R10, R10, 1
    JAL div_loop
    
div_done:
    ; Print tens digit
    ADDI R10, R10, 48  ; Convert to ASCII
    LI R7, 0xFFFF
    STORE R10, R7, 0
    
    ; Print ones digit
    ADDI R11, R11, 48  ; Convert to ASCII
    STORE R11, R7, 0
    
    JAL pn_done
    
print_single_digit:
    ; Single digit
    ADDI R10, R3, 48  ; Convert to ASCII
    LI R7, 0xFFFF
    STORE R10, R7, 0
    
pn_done:
    ; Restore return address and return
    JALR R0, R13, 0