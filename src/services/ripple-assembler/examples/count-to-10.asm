; Count from 1 to 10
; Simple counting example

.data
space: .asciiz " "
newline: .asciiz "\n"
done_msg: .asciiz "Done counting!\n"

.code
start:
    ; Initialize counter to 1
    LI R3, 1
    
count_loop:
    ; Check if we've reached 11
    LI R4, 11
    BEQ R3, R4, counting_done
    
    ; Print the number
    JAL R0, R0, print_number
    
    ; Print space
    LI R4, space
    JAL R0, R0, print_string
    
    ; Increment counter
    ADDI R3, R3, 1
    
    ; Continue loop
    JAL R0, R0, count_loop
    
counting_done:
    ; Print newline
    LI R4, newline
    JAL R0, R0, print_string
    
    ; Print done message
    LI R4, done_msg
    JAL R0, R0, print_string
    
    HALT

; Print number (handles 1-10)
print_number:
    ADD R5, RA, R0  ; Save return address
    
    ; Check if it's 10
    LI R6, 10
    BNE R3, R6, single_digit
    
    ; Print "10"
    LI R6, 49  ; '1'
    STORE R6, R0, 0
    
    LI R6, 48  ; '0'
    STORE R6, R0, 0
    
    JALR R0, R0, R5
    
single_digit:
    ; Convert to ASCII and print
    ADDI R6, R3, 48  ; Add '0'
    STORE R6, R0, 0
    
    JALR R0, R0, R5

; Print string subroutine
print_string:
    ADD R5, RA, R0  ; Save return address
    
str_loop:
    LOAD R6, R0, R4
    BEQ R6, R0, str_done
    
    STORE R6, R0, 0
    
    ADDI R4, R4, 1
    JAL R0, R0, str_loop
    
str_done:
    JALR R0, R0, R5