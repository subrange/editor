; Test branch instructions with labels
_start:
    LI R3, 0       ; counter
    BEQ R0, R0, loop_start
    
loop_start:
    ; Print counter + '0'
    LI R4, 48
    ADD R5, R3, R4
    STORE R5, R0, R0
    
    ; Increment counter
    ADDI R3, R3, 1
    
    ; Check if counter < 3
    LI R6, 3
    SLT R7, R3, R6
    BNE R7, R0, loop_start
    
    ; Done
    LI R8, 10
    STORE R8, R0, R0
    HALT