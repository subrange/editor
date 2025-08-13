; crt0.asm - C Runtime Startup Code
; This is the entry point for C programs
; It initializes the runtime environment and calls main

; Entry point - this is where execution begins
_start:
    ; Initialize stack pointer
    LI SB, 2        ; Stack bank = 1 (SB/R28 - stack bank id)
    LI SP, 4096     ; Stack pointer starts at the end of the bank (SP/R29)
    LI FP, 4096     ; Frame pointer starts at the end of the bank (FP/R30)
    ; Initialize global pointer
    LI GP, 1        ; Global pointer starts at 0 (GP/R31)
    
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