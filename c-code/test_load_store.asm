_start:
    LI R13, 0
    LI R14, 1000
    LI R15, 1000
    CALL _init_data
    CALL main
    HALT

_init_data:
    ; Store 'H' at address 100
    LI R3, 72
    LI R4, 100
    STORE R3, R0, R4
    ; Store 'i' at address 101
    LI R3, 105
    LI R4, 101
    STORE R3, R0, R4
    RET

main:
    ; Load and print from address 100
    LI R3, 100
    LOAD R4, R0, R3
    STORE R4, R0, R0
    ; Load and print from address 101
    LI R3, 101
    LOAD R4, R0, R3
    STORE R4, R0, R0
    ; Newline
    LI R3, 10
    STORE R3, R0, R0
    RET