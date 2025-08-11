; Test comparison operations (SLT, SLTU, BLT, BGE)

_start:
    ; Test SLT (signed less than)
    LI R5, -5       ; Load -5 (0xFFFB in 16-bit)
    LI R6, 3
    SLT R7, R5, R6  ; R7 = 1 (since -5 < 3)
    
    ; Output '0' + 1 = '1'
    LI R8, 48
    ADD R9, R7, R8
    STORE R9, 0, 0
    
    ; Test SLTU (unsigned less than)
    SLTU R10, R5, R6  ; R10 = 0 (since 0xFFFB > 3 unsigned)
    
    ; Output '0' + 0 = '0'
    ADD R11, R10, R8
    STORE R11, 0, 0
    
    ; Test BLT
    LI R12, 5
    LI R13, 10
    BLT R12, R13, less_than
    
    ; Should not execute
    LI R14, 88  ; 'X'
    STORE R14, 0, 0
    
less_than:
    ; Output 'Y' for yes
    LI R15, 89
    STORE R15, 0, 0
    
    ; Test BGE
    BGE R13, R12, greater_equal
    
    ; Should not execute
    LI R5, 78  ; 'N'
    STORE R5, 0, 0
    
greater_equal:
    ; Output 'Y' for yes
    LI R6, 89
    STORE R6, 0, 0
    
    ; Output newline
    LI R7, 10
    STORE R7, 0, 0
    
    HALT