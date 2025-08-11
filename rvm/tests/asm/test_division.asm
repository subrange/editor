; Test division and modulo operations

_start:
    ; Test DIV: 15 / 3 = 5
    LI R5, 15
    LI R6, 3
    DIV R7, R5, R6
    
    ; Output '0' + 5 = '5'
    LI R8, 48
    ADD R9, R7, R8
    STORE R9, 0, 0
    
    ; Test MOD: 17 % 5 = 2
    LI R10, 17
    LI R11, 5
    MOD R12, R10, R11
    
    ; Output '0' + 2 = '2'
    ADD R13, R12, R8
    STORE R13, 0, 0
    
    ; Test DIVI: 21 / 7 = 3
    LI R14, 21
    DIVI R15, R14, 7
    
    ; Output '0' + 3 = '3'
    ADD R5, R15, R8
    STORE R5, 0, 0
    
    ; Test MODI: 10 % 3 = 1
    LI R6, 10
    MODI R7, R6, 3
    
    ; Output '0' + 1 = '1'
    ADD R9, R7, R8
    STORE R9, 0, 0
    
    ; Output newline
    LI R10, 10
    STORE R10, 0, 0
    
    HALT