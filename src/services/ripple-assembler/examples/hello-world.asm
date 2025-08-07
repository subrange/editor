; Hello World Example for Ripple VM
; Demonstrates basic string output

.data
hello_msg:  .asciiz "Hello, World!\n"

;@program_start(@OP_LI,     #R3, 0,   0)     // R3 is loader
;@cmd(          @OP_LI,     #R4, 4,   0)     // jump address, stable
;@cmd(          @OP_LI,     #R5, 2,   0)     // memory pointer

;@cmd(          @OP_JALR,   #R4, 0,   #R4)   // unnecessary, but let it be

;@cmd(          @OP_LOAD,   #R3, 0,   #R5)   // load letter
;@cmd(          @OP_BNE,    #R3, 0,   1)     // if 0 - halt

;@cmd(          @OP_HALT,   0,   0,   0)

;@cmd(          @OP_ADDI,   #R5, #R5, 1)     // inc r5

;@cmd(          @OP_STOR,   #R3, 0,   0)     // send letter to i/o output

;@cmd(          @OP_JALR,   #R4, 0, #R4)

.code
start:
    ; Load address of hello message
    LI R5, hello_msg
    
print_loop:
    ; Load character from string
    LOAD R3, R5, 0
    
    ; Check if null terminator
    BEQ R3, R0, done
    
    ; Output character (I/O is at address 0)
    STORE R3, R0, 0
    
    ; Move to next character
    ADDI R5, R5, 1
    
    ; Continue loop
    JAL print_loop
    
done:
    HALT