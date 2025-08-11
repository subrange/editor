; Simple test without runtime
_start:
    ; Initialize stack like crt0 does
    LI R13, 0
    LI R14, 1000
    LI R15, 1000
    
    ; Call a function
    CALL test_func
    
    ; After return, output 'B'
    LI R5, 66
    STORE R5, 0, 0
    
    HALT

test_func:
    ; Save RA on stack (like compiled code does)
    STORE RA, R13, R14
    ADDI R14, R14, 1
    
    ; Output 'A'
    LI R5, 65
    STORE R5, 0, 0
    
    ; Restore RA from stack
    ADDI R14, R14, -1
    LOAD RA, R13, R14
    
    ; Return
    RET