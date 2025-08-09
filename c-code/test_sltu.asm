; Test SLTU instruction
_start:
    ; Test 1: 0 < 3
    LI R3, 0
    LI R4, 3
    SLTU R5, R3, R4   ; Should be 1
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Print '0' + result
    
    ; Test 2: 3 < 3
    LI R3, 3
    LI R4, 3
    SLTU R5, R3, R4   ; Should be 0
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Print '0' + result
    
    ; Test 3: 5 < 3
    LI R3, 5
    LI R4, 3
    SLTU R5, R3, R4   ; Should be 0
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Print '0' + result
    
    LI R8, 10
    STORE R8, R0, R0  ; Newline
    HALT