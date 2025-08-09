_start:
    LI R13, 0
    LI R14, 1000
    LI R15, 1000
    CALL main
    HALT

main:
    LI R3, 72   ; 'H'
    STORE R3, R0, R0
    LI R3, 105  ; 'i'
    STORE R3, R0, R0
    LI R3, 10   ; '\n'
    STORE R3, R0, R0
    RET