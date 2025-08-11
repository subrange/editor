_start:
    ; Initialize stack  
    LI R13, 0
    LI R14, 100  ; Start at 100 for easier debugging
    LI R15, 100
    
    ; Simulate what crt0 does - call main
    CALL main
    HALT

main:
    ; Simulate what compiled main does
    STORE RA, R13, R14     ; Store RA at stack[100]
    ADDI R14, R14, 1       ; R14 = 101
    STORE R15, R13, R14    ; Store R15 (100) at stack[101]  
    ADDI R14, R14, 1       ; R14 = 102
    ADD R15, R14, R0       ; R15 = 102 (new frame pointer)
    
    ; Do some work
    LI R5, 65
    STORE R5, 0, 0         ; Output 'A'
    
    ; Return sequence
    ADD R14, R15, R0       ; R14 = 102 (restore stack pointer from frame)
    ADDI R14, R14, -1      ; R14 = 101
    LOAD R15, R13, R14     ; Load R15 from stack[101] - should get 100
    ADDI R14, R14, -1      ; R14 = 100
    LOAD RA, R13, R14      ; Load RA from stack[100] - should get return address
    RET