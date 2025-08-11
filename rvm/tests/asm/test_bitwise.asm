; Test bitwise operations

_start:
    ; Test AND
    LI R5, 0xFF
    LI R6, 0x0F
    AND R7, R5, R6   ; R7 = 0x0F = 15
    
    ; Test OR
    LI R8, 0x30     ; '0' = 48
    OR R9, R7, R8   ; R9 = 0x3F = 63 = '?'
    
    ; Test XOR
    LI R10, 0x0C
    XOR R11, R9, R10 ; R11 = 0x33 = 51 = '3'
    
    ; Output '3' to verify XOR result
    STORE R11, 0, 0
    
    ; Test shifts
    LI R12, 1
    LI R13, 3
    SLL R14, R12, R13  ; R14 = 1 << 3 = 8
    
    ; Add to '0' to get '8'
    LI R15, 48
    ADD R5, R14, R15   ; R5 = 8 + 48 = 56 = '8'
    STORE R5, 0, 0
    
    ; Output newline
    LI R6, 10
    STORE R6, 0, 0
    
    HALT