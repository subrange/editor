; FizzBuzz Example for Ripple VM
; Prints numbers 1-100, replacing multiples of 3 with "Fizz",
; multiples of 5 with "Buzz", and multiples of both with "FizzBuzz"

.data
fizz_str:     .asciiz "Fizz"
buzz_str:     .asciiz "Buzz"
fizzbuzz_str: .asciiz "FizzBuzz"
newline:      .asciiz "\n"
space:        .asciiz " "

.code
start:
    ; Initialize counter to 1
    LI R3, 1
    
main_loop:
    ; Check if we've reached 101
    LI R4, 101
    BEQ R3, R4, done
    
    ; Check divisibility by 15 (3*5) first
    ADD R5, R3, R0   ; Copy number
    LI R6, 15
    JALR RA, R0, check_divisible  ; Result in R7
    BEQ R7, R0, print_fizzbuzz
    
    ; Check divisibility by 3
    ADD R5, R3, R0   ; Copy number
    LI R6, 3
    JALR RA, R0, check_divisible  ; Result in R7
    BEQ R7, R0, print_fizz
    
    ; Check divisibility by 5
    ADD R5, R3, R0   ; Copy number
    LI R6, 5
    JALR RA, R0, check_divisible  ; Result in R7
    BEQ R7, R0, print_buzz
    
    ; Not divisible by 3 or 5, print the number
    JALR RA, R3, print_number
    JAL continue_loop

print_fizzbuzz:
    LI R8, fizzbuzz_str
    JALR RA, R8, print_string
    JAL continue_loop

print_fizz:
    LI R8, fizz_str
    JALR RA, R8, print_string
    JAL continue_loop

print_buzz:
    LI R8, buzz_str
    JALR RA, R8, print_string
    JAL continue_loop

continue_loop:
    ; Print space or newline based on position
    ; Print newline every 10 items for readability
    ADD R5, R3, R0   ; Copy counter
    LI R6, 10
    JALR RA, R0, check_divisible
    BEQ R7, R0, print_newline_cont
    
    ; Print space
    LI R8, space
    JALR RA, R8, print_string
    JAL increment_counter

print_newline_cont:
    LI R8, newline
    JALR RA, R8, print_string

increment_counter:
    ; Increment counter
    ADDI R3, R3, 1
    JAL main_loop

done:
    HALT

; Subroutine: check_divisible
; Input: R5 = dividend, R6 = divisor
; Output: R7 = remainder (0 if divisible)
; Uses: R9, R10
check_divisible:
    ADD R9, RA, R0   ; Save return address
    ADD R7, R5, R0   ; Start with dividend as remainder
    
cd_loop:
    SLT R10, R7, R6  ; Check if remainder < divisor
    BNE R10, R0, cd_done
    SUB R7, R7, R6   ; Subtract divisor
    JAL cd_loop
    
cd_done:
    JALR R0, R9, 0   ; Return

; Subroutine: print_string
; Input: R8 = string address
print_string:
    ADD R9, RA, R0   ; Save return address
    
ps2_loop:
    LOAD R10, R8, 0
    BEQ R10, R0, ps2_done
    LI R11, 0xFFFF
    STORE R10, R11, 0
    ADDI R8, R8, 1
    JAL ps2_loop
    
ps2_done:
    JALR R0, R9, 0

; Subroutine: print_number
; Input: R3 = number to print (1-100)
print_number:
    ADD R9, RA, R0   ; Save return address
    
    ; Handle 100 specially
    LI R10, 100
    BNE R3, R10, pn_not_100
    
    ; Print "100"
    LI R10, 49      ; '1'
    LI R11, 0xFFFF
    STORE R10, R11, 0
    LI R10, 48      ; '0'
    STORE R10, R11, 0
    STORE R10, R11, 0
    JALR R0, R9, 0
    
pn_not_100:
    ; Check if >= 10
    LI R10, 10
    SLT R11, R3, R10
    BNE R11, R0, pn_single_digit
    
    ; Two digits - divide by 10
    ADD R12, R3, R0  ; Copy number
    LI R13, 0        ; Tens counter
    
pn_div_loop:
    SLT R11, R12, R10
    BNE R11, R0, pn_div_done
    SUB R12, R12, R10
    ADDI R13, R13, 1
    JAL pn_div_loop
    
pn_div_done:
    ; Print tens digit
    ADDI R13, R13, 48  ; Convert to ASCII
    LI R11, 0xFFFF
    STORE R13, R11, 0
    
    ; Print ones digit
    ADDI R12, R12, 48  ; Convert to ASCII
    STORE R12, R11, 0
    JALR R0, R9, 0
    
pn_single_digit:
    ADDI R10, R3, 48   ; Convert to ASCII
    LI R11, 0xFFFF
    STORE R10, R11, 0
    JALR R0, R9, 0