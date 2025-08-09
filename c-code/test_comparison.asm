; Test comparison instructions
_start:
    ; Test SLTU: 2 < 5
    LI R3, 2
    LI R4, 5
    SLTU R5, R3, R4
    
    ; Print result
    LI R6, 65  ; 'A'
    ADD R7, R6, R5
    STORE R7, R0, R0  ; Should print 'B' if SLTU works
    
    LI R8, 10
    STORE R8, R0, R0
    HALT