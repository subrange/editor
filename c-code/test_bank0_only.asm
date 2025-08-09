_start:
    LI R13, 0
    LI R14, 1000
    LI R15, 1000
    CALL main
    HALT

main:
    ; Store 'H' at address 500 in bank 0
    LI R3, 72
    LI R4, 500
    STORE R3, R0, R4
    ; Load from address 500 in bank 0
    LOAD R5, R0, R4
    ; Output it
    STORE R5, R0, R0
    ; Newline
    LI R3, 10
    STORE R3, R0, R0
    RET