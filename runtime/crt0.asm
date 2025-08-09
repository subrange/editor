; crt0.asm - C Runtime Startup Code
; This is the entry point for C programs
; It initializes the runtime environment and calls main

; Entry point - this is where execution begins
_start:
    ; Initialize stack pointer
    LI R13, 0       ; Stack bank = 0
    LI R14, 1000    ; Stack pointer starts at 1000
    LI R15, 1000    ; Frame pointer starts at 1000
    
    ; TODO: Zero BSS section (uninitialized globals)
    ; For now, we'll skip this as we don't have a BSS section yet
    
    ; TODO: Initialize global variables
    ; This would call _init_globals if we have initialized globals
    
    ; Call main function
    ; main() should be provided by the user program
    CALL main
    
    ; If main returns, halt the program
    ; In a real system, we might call exit() here
    HALT

; Exit function - terminates the program
_exit:
    ; For now, just halt
    HALT