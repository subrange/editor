; crt0.asm - C Runtime Startup Code
; This is the entry point for C programs
; It initializes the runtime environment and calls main

; Entry point - this is where execution begins
_start:
    ; Initialize stack pointer
    LI SB, 1        ; Stack bank = 1 (SB/R28 - stack bank id)
    LI SP, 1000     ; Stack pointer starts at 1000 (SP/R29)
    LI FP, 1000     ; Frame pointer starts at 1000 (FP/R30)
    
    ; TODO: Zero BSS section (uninitialized globals)
    ; For now, we'll skip this as we don't have a BSS section yet
    
    ; Initialize global variables
    ; Call _init_globals if it exists (the linker will resolve this)
    CALL _init_globals
    
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