; Manual while loop to debug
_start:
    ; Initialize i = 0
    LI R13, 0       ; Stack bank
    LI R14, 1000   ; Stack pointer  
    LI R15, 1000   ; Frame pointer
    
    ADDI R3, R15, 1  ; Address for i (FP+1)
    LI R4, 0         ; i = 0
    STORE R4, R13, R3 ; Store i on stack
    
    ; Print 'S:'
    LI R5, 83
    STORE R5, R0, R0
    LI R5, 58
    STORE R5, R0, R0
    
loop_start:
    ; Load i from stack
    ADDI R3, R15, 1
    LOAD R6, R13, R3
    
    ; Print 'L' to show we're in loop check
    LI R7, 76
    STORE R7, R0, R0
    
    ; Compare i < 3
    LI R7, 3
    SLTU R8, R6, R7
    
    ; Print comparison result ('0' + result)
    LI R9, 48
    ADD R10, R8, R9
    STORE R10, R0, R0
    
    ; Branch if i < 3
    BNE R8, R0, loop_body
    BEQ R0, R0, loop_end
    
loop_body:
    ; Print 'B' for body
    LI R7, 66
    STORE R7, R0, R0
    
    ; Print i value
    ADDI R3, R15, 1
    LOAD R6, R13, R3
    LI R7, 48
    ADD R8, R6, R7
    STORE R8, R0, R0
    
    ; Increment i
    ADDI R3, R15, 1
    LOAD R6, R13, R3
    ADDI R6, R6, 1
    STORE R6, R13, R3
    
    ; Jump back to loop start
    BEQ R0, R0, loop_start
    
loop_end:
    ; Print 'E'
    LI R3, 69
    STORE R3, R0, R0
    LI R4, 10
    STORE R4, R0, R0
    
    HALT