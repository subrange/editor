; Test memory operations (LOAD/STORE)

_start:
    ; Store values to memory
    LI R5, 72      ; 'H'
    STORE R5, 0, 10  ; Store at address 10
    
    LI R6, 105     ; 'i'
    STORE R6, 0, 11  ; Store at address 11
    
    ; Load values back
    LOAD R7, 0, 10
    LOAD R8, 0, 11
    
    ; Output loaded values
    STORE R7, 0, 0  ; Output 'H'
    STORE R8, 0, 0  ; Output 'i'
    
    ; Output newline
    LI R9, 10
    STORE R9, 0, 0
    
    HALT