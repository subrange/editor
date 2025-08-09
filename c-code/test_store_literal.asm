_start:
    LI R13, 0
    LI R14, 1000
    LI R15, 1000
    CALL main
    HALT

main:
    ; Store 'H' at address 100 using literal 0 for bank
    LI R3, 72
    LI R4, 100
    STORE R3, 0, R4
    ; Load it back
    LOAD R5, 0, R4
    ; Output it
    STORE R5, 0, 0
    ; Newline
    LI R3, 10
    STORE R3, 0, 0
    RET