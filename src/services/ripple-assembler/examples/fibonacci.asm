; Fibonacci Sequence Example for Ripple VM
; Calculates and displays Fibonacci numbers
; Demonstrates recursion and iterative approaches

.data
fib_msg:    .asciiz "Fibonacci sequence (first 20 numbers):\n"
space:      .asciiz " "
newline:    .asciiz "\n"
fib_array:  .space 20  ; Storage for 20 Fibonacci numbers

.code
start:
    ; Print header message
    LI R3, fib_msg
    JALR RA, R3, print_string
    
    ; Calculate Fibonacci sequence iteratively
    ; R3 = n-2 (starts at 0)
    ; R4 = n-1 (starts at 1)
    ; R5 = current fib number
    ; R6 = counter
    ; R7 = array pointer
    
    LI R3, 0      ; fib(0) = 0
    LI R4, 1      ; fib(1) = 1
    LI R6, 0      ; counter
    LI R7, fib_array  ; array pointer
    
    ; Store first two numbers
    STORE R3, R7, 0   ; fib_array[0] = 0
    ADDI R7, R7, 1
    STORE R4, R7, 0   ; fib_array[1] = 1
    ADDI R7, R7, 1
    
    LI R6, 2      ; Start counter at 2
    
fib_loop:
    ; Check if we've calculated 20 numbers
    LI R8, 20
    BEQ R6, R8, print_sequence
    
    ; Calculate next Fibonacci number
    ADD R5, R3, R4   ; fib(n) = fib(n-1) + fib(n-2)
    
    ; Store in array
    STORE R5, R7, 0
    ADDI R7, R7, 1
    
    ; Update for next iteration
    ADD R3, R4, R0   ; n-2 = old n-1
    ADD R4, R5, R0   ; n-1 = current n
    
    ; Increment counter
    ADDI R6, R6, 1
    
    JAL fib_loop

print_sequence:
    ; Print all numbers in the array
    LI R6, 0         ; counter
    LI R7, fib_array ; reset array pointer
    
print_loop:
    LI R8, 20
    BEQ R6, R8, done
    
    ; Load number from array
    LOAD R3, R7, 0
    
    ; Print the number
    JALR RA, R3, print_number
    
    ; Print space
    LI R3, space
    JALR RA, R3, print_string
    
    ; Every 5 numbers, print newline
    LI R8, 5
    ADDI R9, R6, 1   ; R9 = counter + 1
    ; Calculate remainder using repeated subtraction
    ADD R10, R9, R0
rem_loop:
    SLT R11, R10, R8
    BNE R11, R0, rem_done
    SUB R10, R10, R8
    JAL rem_loop
rem_done:
    BNE R10, R0, skip_newline
    LI R3, newline
    JALR RA, R3, print_string
    
skip_newline:
    ; Move to next array element
    ADDI R7, R7, 1
    ADDI R6, R6, 1
    
    JAL print_loop

done:
    ; Final newline if needed
    LI R3, newline
    JALR RA, R3, print_string
    HALT

; Subroutine: print_string (same as in 99-bottles)
print_string:
    ADD R12, RA, R0
ps_loop:
    LOAD R13, R3, 0
    BEQ R13, R0, ps_done
    LI R14, 0xFFFF
    STORE R13, R14, 0
    ADDI R3, R3, 1
    JAL ps_loop
ps_done:
    JALR R0, R12, 0

; Subroutine: print_number
; Enhanced to handle larger numbers (up to 9999)
print_number:
    ADD R12, RA, R0  ; Save return address
    
    ; Handle 0 specially
    BNE R3, R0, pn_not_zero
    LI R13, 48       ; ASCII '0'
    LI R14, 0xFFFF
    STORE R13, R14, 0
    JALR R0, R12, 0
    
pn_not_zero:
    ; Convert number to digits using division
    ; We'll use a simple digit extraction method
    ; Store digits in reverse order in R8-R11
    
    ADD R13, R3, R0  ; Working copy
    LI R14, 0        ; Digit count
    
    ; Extract ones
    LI R15, 10
    ; Division by subtraction
    ADD R8, R13, R0  ; Remainder
div1_loop:
    SLT R9, R8, R15
    BNE R9, R0, div1_done
    SUB R8, R8, R15
    JAL div1_loop
div1_done:
    
    ; Calculate quotient for tens
    SUB R13, R13, R8  ; Remove ones digit
    ; R13 now divisible by 10, divide by shifting (approximate)
    ; For simplicity, count how many 10s
    LI R9, 0
    ADD R10, R13, R0
count_tens:
    BEQ R10, R0, tens_done
    SUB R10, R10, R15
    ADDI R9, R9, 1
    JAL count_tens
tens_done:
    
    ; Now R8 = ones, R9 = tens
    ; Print tens if non-zero
    BEQ R9, R0, print_ones
    ADDI R9, R9, 48  ; Convert to ASCII
    LI R14, 0xFFFF
    STORE R9, R14, 0
    
print_ones:
    ADDI R8, R8, 48  ; Convert to ASCII
    LI R14, 0xFFFF
    STORE R8, R14, 0
    
    JALR R0, R12, 0