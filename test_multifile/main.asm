; Main file that calls a function from another file
_start:
    ; Call function from other file
    CALL print_char
    
    ; Print newline
    LI R5, 10
    STORE R5, 0, 0
    
    HALT