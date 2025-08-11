; Test arithmetic and control flow instructions
; This program computes 2 + 3 = 5 and outputs '5' (ASCII 53)

_start:
    ; Test ADD instruction
    LI R5, 2
    LI R6, 3
    ADD R7, R5, R6    ; R7 = 2 + 3 = 5
    
    ; Test multiplication
    LI R8, 10
    MUL R9, R7, R8    ; R9 = 5 * 10 = 50
    
    ; Add 3 to get ASCII '5' (53)
    LI R10, 3
    ADD R11, R9, R10  ; R11 = 50 + 3 = 53
    
    ; Output the result
    STORE R11, 0, 0   ; Output '5'
    
    ; Test branching
    LI R12, 1
    LI R13, 1
    BEQ R12, R13, skip_error
    
    ; This shouldn't execute
    LI R14, 88       ; 'X' for error
    STORE R14, 0, 0
    
skip_error:
    ; Output newline
    LI R15, 10
    STORE R15, 0, 0
    
    ; Test loop - output 3 dots
    LI R5, 3         ; Counter
    LI R6, 46        ; '.' ASCII
    
loop:
    STORE R6, 0, 0   ; Output dot
    ADDI R5, R5, -1  ; Decrement counter (add -1)
    BNE R5, R0, loop ; Continue if not zero
    
    ; Final newline
    LI R7, 10
    STORE R7, 0, 0
    
    HALT