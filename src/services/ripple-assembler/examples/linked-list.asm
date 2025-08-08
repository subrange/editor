; Linked List Implementation for Ripple VM
; Demonstrates dynamic data structures, memory management, and pointers
; Each node contains: value (1 word) + next pointer (1 word)

.data
heap_start:   .space 100   ; Simple heap for allocating nodes
heap_ptr:     .word 0      ; Current heap pointer (offset from heap_start)
list_head:    .word 0      ; Head of the linked list
msg_empty:    .asciiz "List is empty\n"
msg_contents: .asciiz "List contents: "
arrow:        .asciiz " -> "
null_str:     .asciiz "NULL\n"
newline:      .asciiz "\n"
msg_added:    .asciiz "Added: "
msg_removed:  .asciiz "Removed: "
msg_length:   .asciiz "List length: "

.code
start:
    ; Initialize heap pointer
    LI R3, heap_start
    LI R4, heap_ptr
    STORE R3, R4, 0
    
    ; Initialize list head to NULL (0)
    LI R3, 0
    LI R4, list_head
    STORE R3, R4, 0
    
    ; Build a sample list: 10 -> 20 -> 30 -> 40
    LI R3, 10
    JAL R0, R0, list_push
    
    LI R3, 20
    JAL R0, R0, list_push
    
    LI R3, 30
    JAL R0, R0, list_push
    
    LI R3, 40
    JAL R0, R0, list_push
    
    ; Print the list
    JAL R0, R0, list_print
    
    ; Get list length
    JAL R0, R0, list_length
    ; R3 now contains length
    LI R4, msg_length
    JAL R0, R0, print_string
    JAL R0, R0, print_number
    LI R4, newline
    JAL R0, R0, print_string
    
    ; Remove first element
    JAL R0, R0, list_pop
    ; R3 contains popped value (or -1 if empty)
    LI R4, msg_removed
    JAL R0, R0, print_string
    JAL R0, R0, print_number
    LI R4, newline
    JAL R0, R0, print_string
    
    ; Print list again
    JAL R0, R0, list_print
    
    ; Add 50 to the end
    LI R3, 50
    JAL R0, R0, list_append
    
    ; Print final list
    JAL R0, R0, list_print
    
    HALT

; Subroutine: allocate_node
; Returns: R3 = address of new node (or 0 if out of memory)
; Uses: R4, R5, R6
allocate_node:
    ADD R6, RA, R0   ; Save return address
    
    ; Load current heap pointer
    LI R4, heap_ptr
    LOAD R3, R4, 0
    
    ; Check if we have space (heap_start + 100 words max)
    LI R5, heap_start
    ADD R5, R5, R3   ; Current allocation address
    
    ; Simple bounds check - assuming heap < 98 words used
    LI R4, 98
    SLT R4, R3, R4
    BEQ R4, R0, alloc_fail
    
    ; Update heap pointer (advance by 2 words)
    ADDI R3, R3, 2
    LI R4, heap_ptr
    STORE R3, R4, 0
    
    ; Return address of allocated node
    ADD R3, R5, R0
    JALR R0, R0, R6
    
alloc_fail:
    LI R3, 0         ; Return NULL
    JALR R0, R0, R6

; Subroutine: list_push (add to beginning)
; Input: R3 = value to add
; Uses: R4, R5, R7, R8
list_push:
    ADD R7, RA, R0   ; Save return address
    ADD R8, R3, R0   ; Save value
    
    ; Allocate new node
    JAL R0, R0, allocate_node
    BEQ R3, R0, push_done  ; Allocation failed
    
    ; Store value in node
    STORE R8, R3, 0
    
    ; Load current head
    LI R4, list_head
    LOAD R5, R4, 0
    
    ; Store current head as next pointer
    STORE R5, R3, 1
    
    ; Update head to new node
    STORE R3, R4, 0
    
    ; Print confirmation
    LI R4, msg_added
    JAL R0, R0, print_string
    JAL R0, R0, print_number
    LI R4, newline
    JAL R0, R0, print_string
    
push_done:
    JALR R0, R0, R7

; Subroutine: list_pop (remove from beginning)
; Output: R3 = value removed (or -1 if empty)
; Uses: R4, R5, R6
list_pop:
    ADD R6, RA, R0   ; Save return address
    
    ; Load head
    LI R4, list_head
    LOAD R5, R4, 0
    
    ; Check if empty
    BEQ R5, R0, pop_empty
    
    ; Load value from head node
    LOAD R3, R5, 0
    
    ; Load next pointer
    LOAD R5, R5, 1
    
    ; Update head to next
    STORE R5, R4, 0
    
    JALR R0, R0, R6
    
pop_empty:
    LI R3, -1        ; Return -1 for empty
    JALR R0, R0, R6

; Subroutine: list_append (add to end)
; Input: R3 = value to add
; Uses: R4, R5, R7, R8, R9
list_append:
    ADD R7, RA, R0   ; Save return address
    ADD R8, R3, R0   ; Save value
    
    ; Allocate new node
    JAL R0, R0, allocate_node
    BEQ R3, R0, append_done  ; Allocation failed
    
    ; Store value and NULL next pointer
    STORE R8, R3, 0
    LI R4, 0
    STORE R4, R3, 1
    
    ADD R9, R3, R0   ; Save new node address
    
    ; Find end of list
    LI R4, list_head
    LOAD R5, R4, 0
    
    ; If list empty, make this the head
    BEQ R5, R0, append_as_head
    
    ; Traverse to end
find_end:
    LOAD R4, R5, 1   ; Load next pointer
    BEQ R4, R0, found_end
    ADD R5, R4, R0   ; Move to next node
    JAL R0 R0 find_end
    
found_end:
    ; R5 points to last node
    STORE R9, R5, 1  ; Set its next to new node
    
    ; Print confirmation
    LI R4, msg_added
    JAL R0, R0, print_string
    JAL R0, R0, print_number
    LI R4, newline
    JAL R0, R0, print_string
    
    JALR R0, R0, R7
    
append_as_head:
    LI R4, list_head
    STORE R9, R4, 0
    
    ; Print confirmation
    LI R4, msg_added
    JAL R0, R0, print_string
    JAL R0, R0, print_number
    LI R4, newline
    JAL R0, R0, print_string
    
append_done:
    JALR R0, R0, R7

; Subroutine: list_length
; Output: R3 = length of list
; Uses: R4, R5, R6
list_length:
    ADD R6, RA, R0   ; Save return address
    
    LI R3, 0         ; Counter
    LI R4, list_head
    LOAD R5, R4, 0   ; Current node
    
len_loop:
    BEQ R5, R0, len_done
    ADDI R3, R3, 1
    LOAD R5, R5, 1   ; Next node
    JAL R0 R0 len_loop
    
len_done:
    JALR R0, R0, R6

; Subroutine: list_print
; Uses: R4, R5, R6
list_print:
    ADD R6, RA, R0   ; Save return address
    
    ; Print header
    LI R4, msg_contents
    JAL R0, R0, print_string
    
    ; Load head
    LI R4, list_head
    LOAD R5, R4, 0
    
    ; Check if empty
    BNE R5, R0, print_loop
    LI R4, msg_empty
    JAL R0, R0, print_string
    JALR R0, R0, R6
    
print_loop:
    ; Print value
    LOAD R3, R5, 0
    JAL R0, R0, print_number
    
    ; Load next
    LOAD R5, R5, 1
    
    ; Check if more nodes
    BEQ R5, R0, print_null
    
    ; Print arrow
    LI R4, arrow
    JAL R0, R0, print_string
    JAL R0 R0 print_loop
    
print_null:
    ; Print NULL
    LI R4, arrow
    JAL R0, R0, print_string
    LI R4, null_str
    JAL R0, R0, print_string
    JALR R0, R0, R6

; Utility subroutines (same as other examples)
print_string:
    ADD R10, RA, R0
ps3_loop:
    LOAD R11, R4, 0
    BEQ R11, R0, ps3_done
    LI R12, 0xFFFF
    STORE R11, R12, 0
    ADDI R4, R4, 1
    JAL R0 R0 ps3_loop
ps3_done:
    JALR R0, R0, R10

print_number:
    ADD R10, RA, R0
    
    ; Simple two-digit printer
    LI R11, 10
    SLT R12, R3, R11
    BNE R12, R0, pn2_single
    
    ; Two digits
    ADD R13, R3, R0
    LI R14, 0
pn2_div:
    SLT R12, R13, R11
    BNE R12, R0, pn2_print_tens
    SUB R13, R13, R11
    ADDI R14, R14, 1
    JAL R0 R0 pn2_div
    
pn2_print_tens:
    ADDI R14, R14, 48
    LI R12, 0xFFFF
    STORE R14, R12, 0
    ADDI R13, R13, 48
    STORE R13, R12, 0
    JALR R0, R0, R10
    
pn2_single:
    ADDI R11, R3, 48
    LI R12, 0xFFFF
    STORE R11, R12, 0
    JALR R0, R0, R10