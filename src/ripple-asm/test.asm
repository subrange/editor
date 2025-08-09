; Test assembly file
.data
hello: .asciiz "Hello, World!\n"

.code
_start:
    LI R3, 0        ; Initialize counter
    LI R5, 2        ; Data pointer
    
print_loop:
    LOAD R4, R5, 0  ; Load character
    BEQ R4, R0, done ; If null, we're done
    STORE R4, R0, 0 ; Output character
    INC R5          ; Move to next character
    JAL R0, R0, print_loop
    
done:
    HALT