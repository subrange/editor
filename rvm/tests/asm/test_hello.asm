; Simple test program that outputs "Hello"
; This tests basic instructions like LI, STORE, and HALT

_start:
    ; Load 'H' (72) and output it
    LI R5, 72
    STORE R5, 0, 0   ; Store to output register
    
    ; Load 'e' (101) and output it
    LI R5, 101
    STORE R5, 0, 0
    
    ; Load 'l' (108) and output it
    LI R5, 108
    STORE R5, 0, 0
    
    ; Output another 'l'
    STORE R5, 0, 0
    
    ; Load 'o' (111) and output it
    LI R5, 111
    STORE R5, 0, 0
    
    ; Load newline (10) and output it
    LI R5, 10
    STORE R5, 0, 0
    
    ; Halt the program
    HALT