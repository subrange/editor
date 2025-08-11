; Test jump instructions (CALL/RET which expand to JAL/JALR)

_start:
    ; Test CALL (expands to JAL RA, R0, func1)
    CALL func1
    
    ; After return, output 'B'
    LI R5, 66
    STORE R5, 0, 0
    
    ; Test another CALL
    CALL func2
    
    ; After second return, output 'D'
    LI R5, 68
    STORE R5, 0, 0
    
    ; Output newline and halt
    LI R5, 10
    STORE R5, 0, 0
    HALT

func1:
    ; Output 'A'
    LI R5, 65
    STORE R5, 0, 0
    ; Return (expands to JALR R0, R0, RA)
    RET

func2:
    ; Output 'C'
    LI R5, 67
    STORE R5, 0, 0
    ; Return
    RET