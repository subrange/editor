; Test SLTU with stack value
_start:
    ; Setup stack
    LI R13, 0       ; Stack bank
    LI R14, 1000   ; Stack pointer  
    LI R15, 1000   ; Frame pointer
    
    ; Store 0 on stack
    ADDI R3, R15, 1
    LI R4, 0
    STORE R4, R13, R3
    
    ; Load from stack
    ADDI R3, R15, 1
    LOAD R5, R13, R3
    
    ; Print loaded value ('0' + value)
    LI R6, 48
    ADD R7, R5, R6
    STORE R7, R0, R0  ; Should print '0'
    
    ; Compare with 3
    LI R8, 3
    SLTU R9, R5, R8
    
    ; Print comparison result
    LI R10, 48
    ADD R11, R9, R10
    STORE R11, R0, R0  ; Should print '1'
    
    LI R12, 10
    STORE R12, R0, R0
    HALT