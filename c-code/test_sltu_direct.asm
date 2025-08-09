; Test SLTU directly
_start:
    ; Test 0 < 3
    LI R3, 0
    LI R4, 3
    SLTU R5, R3, R4
    
    ; Print result
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Should print '1'
    
    ; Test 1 < 3
    LI R3, 1
    LI R4, 3
    SLTU R5, R3, R4
    
    ; Print result
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Should print '1'
    
    ; Test 2 < 3
    LI R3, 2
    LI R4, 3
    SLTU R5, R3, R4
    
    ; Print result
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Should print '1'
    
    ; Test 3 < 3
    LI R3, 3
    LI R4, 3
    SLTU R5, R3, R4
    
    ; Print result
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Should print '0'
    
    LI R8, 10
    STORE R8, R0, R0
    HALT