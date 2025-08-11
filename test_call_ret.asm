_start:
    LI R13, 0
    LI R14, 100
    LI R15, 100
    
    ; Debug: Store a marker value at stack[100] before CALL
    LI R5, 999
    STORE R5, R13, R14
    
    CALL test_func
    
    ; After return, output 'B'
    LI R5, 66
    STORE R5, 0, 0
    HALT

test_func:
    ; Output 'A'
    LI R5, 65
    STORE R5, 0, 0
    RET