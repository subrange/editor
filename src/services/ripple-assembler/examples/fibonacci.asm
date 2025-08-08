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
    JAL R0, R0, print_string
    
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
    STORE R3, 0, R7   ; fib_array[0] = 0
    ADDI R7, R7, 1
    STORE R4, 0, R7   ; fib_array[1] = 1
    ADDI R7, R7, 1

    LI R6, 2      ; Start counter at 2

fib_loop:
    ; Check if we've calculated 20 numbers
    LI R8, 20
    BEQ R6, R8, print_sequence

    ; Calculate next Fibonacci number
    ADD R5, R3, R4   ; fib(n) = fib(n-1) + fib(n-2)

    ; Store in array
    STORE R5, 0, R7
    ADDI R7, R7, 1

    ; Update for next iteration
    ADD R3, R4, R0   ; n-2 = old n-1
    ADD R4, R5, R0   ; n-1 = current n

    ; Increment counter
    ADDI R6, R6, 1

    JAL R0 R0 fib_loop

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
    JAL R0, R0, print_number

    ; Print space
    LI R3, space
    JAL R0, R0, print_string

    ; Every 5 numbers, print newline
    LI R8, 5
    ADDI R9, R6, 1   ; R9 = counter + 1
    ; Calculate remainder using repeated subtraction
    ADD R10, R9, R0
rem_loop:
    BLT R10, R8, rem_done
    SUB R10, R10, R8
    JAL R0 R0 rem_loop
rem_done:
    BNE R10, R0, skip_newline
    LI R3, newline
    JAL R0, R0, print_string

skip_newline:
    ; Move to next array element
    ADDI R7, R7, 1
    ADDI R6, R6, 1

    JAL R0 R0 print_loop

done:
    ; Final newline if needed
    LI R3, newline
    JAL R0, R0, print_string
    HALT

; Subroutine: print_string (same as in 99-bottles)
print_string:
    ADD R12, RA, R0
ps_loop:
    LOAD R13, R3, 0
    BEQ R13, R0, ps_done
    STORE R13, 0, 0
    ADDI R3, R3, 1
    JAL R0 R0 ps_loop
ps_done:
    JALR R0, R0, R12

; Subroutine: print_number
; Input: R3 = non-negative integer (0..9999). Prints decimal to OUT.
print_number:
    ADD R12, RA, R0          ; save return

    ; if R3 == 0 -> "0"
    BNE R3, R0, pn_nonzero
    LI  R13, 48
    STORE R13, R0, 0
    JALR R0, R0, R12

pn_nonzero:
    LI  R14, 0               ; printed_any = 0
    ADD R15, R3, R0          ; n = R3

    ; thousands
    LI  R10, 1000
    LI  R11, 0               ; d = 0
pn_t_loop:
    BLT R15, R10, pn_t_done
    SUB R15, R15, R10
    ADDI R11, R11, 1
    JAL  R0, R0, pn_t_loop
pn_t_done:
    BEQ R11, R0, pn_hundreds
    ADDI R11, R11, 48
    STORE R11, R0, 0
    LI   R14, 1

    ; hundreds
pn_hundreds:
    LI  R10, 100
    LI  R11, 0
pn_h_loop:
    BLT R15, R10, pn_h_done
    SUB R15, R15, R10
    ADDI R11, R11, 1
    JAL  R0, R0, pn_h_loop
pn_h_done:
    BEQ R11, R0, pn_tens_check
    ADDI R11, R11, 48
    STORE R11, R0, 0
    LI   R14, 1

    ; tens
pn_tens_check:
    LI  R10, 10
    LI  R11, 0
pn_te_loop:
    BLT R15, R10, pn_te_done
    SUB R15, R15, R10
    ADDI R11, R11, 1
    JAL  R0, R0, pn_te_loop
pn_te_done:
    ; print tens if already printed something OR tens>0
    BEQ R14, R0, pn_tens_if_nonzero
    ; already printed higher digit -> must print tens even if 0
    ADDI R11, R11, 48
    STORE R11, R0, 0
    JAL  R0, R0, pn_ones
pn_tens_if_nonzero:
    BEQ R11, R0, pn_ones
    ADDI R11, R11, 48
    STORE R11, R0, 0
    LI   R14, 1

    ; ones (R15 is now 0..9)
pn_ones:
    ADDI R15, R15, 48
    STORE R15, R0, 0
    JALR R0, R0, R12