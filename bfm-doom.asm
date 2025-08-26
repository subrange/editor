












































































li  sb , 2 
li  sp , 1 
li  fp , 1 
li  gp , 1 
call  main 
HALT





































storage_set_block:
 li  sc , 17 
store a0 , r0 , sc
 jalr rab, rab, ra 

storage_set_addr:
 li  sc , 18 
store a0 , r0 , sc
 jalr rab, rab, ra 

storage_read:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  sc , 19 
load rv0 , r0 , sc 
add  rv0 , rv0 , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

storage_get_status:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  sc , 20 
load rv0 , r0 , sc
addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

storage_write:
 li  sc , 19 
store a0 , r0 , sc
 jalr rab, rab, ra 

storage_commit:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  sc , 0x4 
li  sc , 20 
store sc , r0 , sc 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

storage_commit_all:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  sc , 0x8 
li  sc , 20 
store sc , r0 , sc 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

storage_write_at:
 add  a0 , a0 , r0 
call  storage_set_block 
add  a0 , a1 , r0 
call  storage_set_addr 
add  a0 , a2 , r0 
call  storage_write
 jalr rab, rab, ra 

storage_read_at:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  a0 , a0 , r0 
call  storage_set_block 
add  a0 , a1 , r0 
call  storage_set_addr 
call  storage_read 
add  rv0 , rv0 , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

storage_read_at_le_unpack:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  a0 , a0 , r0 
call  storage_set_block 
add  a0 , a1 , r0 
call  storage_set_addr 
call  storage_read 
andi  rv1 ,  rv0 ,  65280 
srli  rv1 ,  rv1 ,  8 
andi  rv0 ,  rv0 ,  255 
add  rv0 , rv0 , r0 
add  rv1 , rv1 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 


storage_read_u16_le:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
li  s2 , 0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
 add  s2 , rv0 , r0 
addi  s1 ,  s1 ,  1 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
slli  rv0 ,  rv0 ,  8 
or  s2 ,  s2 ,  rv0 
add  rv0 , s2 , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

storage_read_u32_le:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
li  s2 , 0 
li  s3 , 0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_u16_le 
 add  s2 , rv0 , r0 
addi  s1 ,  s1 ,  2 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_u16_le 
 add  s3 , rv0 , r0 
add  rv0 , s3 , r0 
add  rv1 , s2 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 



graphics_flush:
 li  sc , 1 
li  sc , 9 
store sc , r0 , sc
 jalr rab, rab, ra 

putchar:
 li  sc , 0 
store a0 , r0 , sc
 jalr rab, rab, ra 


puts:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  t0 , a1 , r0 
add  t1 , a0 , r0 
load sc , t1 , t0  
loop_1:
bne  sc ,  r0 , cont_2
beq r0, r0, end_3
cont_2: 
add  a0 , sc , r0 
call  putchar 
addi  t0 ,  t0 ,  1 
load sc , t1 , t0 
beq r0, r0, loop_1
end_3: 
li  a0 , 10 
call  putchar 
li  sc , 0 
add  rv0 , sc , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

digit_hex_ascii:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  sc , 10 
sltu  sc, ,  a0 ,  sc 
bne sc , r0,efelsetruebranch_4
ifelsefalsebranch_5: 
addi  rv0 ,  a0 ,  55 
beq r0, r0,  endifelse_6 

efelsetruebranch_4: 
addi  rv0 ,  a0 ,  48 
endifelse_6: 
add  rv0 , rv0 , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 


print_hex16:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  t0 , a0 , r0 
li  a0 , 48 
call  putchar 
li  a0 , 120 
call  putchar 
srli sc , t0 , 12 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 
srli sc , t0 , 8 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 
srli sc , t0 , 4 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 
srli sc , t0 , 0 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

print_hex16_no_prefix:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  t0 , a0 , r0 
srli sc , t0 , 12 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 
srli sc , t0 , 8 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 
srli sc , t0 , 4 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 
srli sc , t0 , 0 
andi sc, sc, 0xF 
add  a0 , sc , r0 
call  digit_hex_ascii 
add  a0 , rv0 , r0 
call  putchar 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

print_hex32:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  t0 , a0 , r0 
add  t1 , a1 , r0 
add  a0 , t0 , r0 
call  print_hex16 
add  a0 , t1 , r0 
call  print_hex16_no_prefix 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

print_digit:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  sc , 10 
sltu  sc, ,  a0 ,  sc 
bne sc , r0,efelsetruebranch_7
ifelsefalsebranch_8: 
li  sc , 63 
add  a0 , sc , r0 
call  putchar 
beq r0, r0,  endifelse_9 

efelsetruebranch_7: 
addi  sc ,  a0 ,  48 
add  a0 , sc , r0 
call  putchar 
endifelse_9: 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 


print_uint:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  t0 , a0 , r0 
li  sc , 10 
sltu  sc, ,  t0 ,  sc 
bne sc , r0,iftruebranch_11
beq r0, r0,  endif_12 

iftruebranch_11: 
beq r0, r0,  units_10 
endif_12: 
li  sc , 100 
sltu  sc, ,  t0 ,  sc 
bne sc , r0,iftruebranch_14
beq r0, r0,  endif_15 

iftruebranch_14: 
beq r0, r0,  tens_13 
endif_15: 
li  sc , 1000 
sltu  sc, ,  t0 ,  sc 
bne sc , r0,iftruebranch_17
beq r0, r0,  endif_18 

iftruebranch_17: 
beq r0, r0,  hundreds_16 
endif_18: 
li  sc , 10000 
sltu  sc, ,  t0 ,  sc 
bne sc , r0,iftruebranch_20
beq r0, r0,  endif_21 

iftruebranch_20: 
beq r0, r0,  thousands_19 
endif_21: 
beq r0, r0,  tens_of_thousands_22 
tens_of_thousands_22: 
divi  sc ,  t0 ,  10000 
add  a0 , sc , r0 
call  print_digit 
thousands_19: 
divi  sc ,  t0 ,  1000 
modi  sc ,  sc ,  10 
add  a0 , sc , r0 
call  print_digit 
hundreds_16: 
divi  sc ,  t0 ,  100 
modi  sc ,  sc ,  10 
add  a0 , sc , r0 
call  print_digit 
tens_13: 
divi  sc ,  t0 ,  10 
modi  sc ,  sc ,  10 
add  a0 , sc , r0 
call  print_digit 
units_10: 
modi  sc ,  t0 ,  10 
add  a0 , sc , r0 
call  print_digit 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 



get_titlepic_table_pos:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s2 , a0 , r0 
; "Lump count" 
li  s3 , 0 
li  a0 , 0 
li  a1 , 8 
call  storage_read_u32_le 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  print_hex32 
li  t0 , 8 
loop_24:
bne  s2 ,  s3 , cont_25
beq r0, r0, end_26
cont_25: 
muli  t1 ,  s3 ,  16 
addi  t0 ,  t1 ,  8 
add  t0 ,  t0 ,  s1 
add  a0 , s0 , r0 
add  a1 , t0 , r0 
call  storage_read_at 
 add  x0 , rv0 , r0 
addi  t0 ,  t0 ,  1 
add  a0 , s0 , r0 
add  a1 , t0 , r0 
call  storage_read_at 
 add  x1 , rv0 , r0 
xori  x0 ,  x0 ,  84 
; if (x0 == 'T' && x1 == 'I') 
xori  x1 ,  x1 ,  73 
or  x0 ,  x0 ,  x1 
beq  x0 ,  r0 ,  end_23 
addi  s3 ,  s3 ,  1 
beq r0, r0, loop_24
end_26: 
end_23:addi  t0 ,  t0 ,  -5 
add  rv0 , s0 , r0 
add  rv1 , t0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

get_titlepic_header_pos:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
addi  s1 ,  s1 ,  -4 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_u32_le 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
add  rv0 , s0 , r0 
add  rv1 , s1 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

iwad:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  a0 , 0 
li  a1 , 0 
call  storage_read_at 
add  a0 , rv0 , r0 
call  putchar 
li  a0 , 0 
li  a1 , 1 
call  storage_read_at 
add  a0 , rv0 , r0 
call  putchar 
li  a0 , 0 
li  a1 , 2 
call  storage_read_at 
add  a0 , rv0 , r0 
call  putchar 
li  a0 , 0 
li  a1 , 3 
call  storage_read_at 
add  a0 , rv0 , r0 
call  putchar 
li  x0 , 10 
li  sc , 0 
store x0 , r0 , sc 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

get_image_size:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_u16_le 
 add  x0 , rv0 , r0 
addi  s1 ,  s1 ,  2 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_u16_le 
 add  x1 , rv0 , r0 
add  rv0 , x0 , r0 
add  rv1 , x1 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

add_u32:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  4 
    addi  sc ,  fp ,  0 
 store a0 , sb , sc 
addi  sc ,  fp ,  1 
 store a1 , sb , sc 
addi  sc ,  fp ,  2 
 store a2 , sb , sc 
addi  sc ,  fp ,  3 
 store a3 , sb , sc 
add  rv1 ,  a1 ,  a3 
sltu  sc, ,  rv1 ,  a1 
add  rv0 ,  a0 ,  a2 
add  rv0 ,  rv0 ,  sc 
add  rv0 , rv0 , r0 
add  rv1 , rv1 , r0 

addi  sp ,  sp ,  -4 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

set_image_byte:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
add  s2 , a2 , r0 
muli  s3 ,  s0 ,  200 
add  s3 ,  s3 ,  s1 
li  s1 , 3 
store s2 , s1 , s3 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

read_column:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  9 
    addi  sc ,  fp ,  0 
 store a0 , sb , sc 
addi  sc ,  fp ,  1 
 store a1 , sb , sc 
addi  sc ,  fp ,  2 
 store a2 , sb , sc 
addi  sc ,  fp ,  3 
 store a3 , sb , sc 
add  a0 , a0 , r0 
add  a1 , a1 , r0 
add  a2 , a2 , r0 
add  a3 , a3 , r0 
call  add_u32 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
addi  sc ,  fp ,  4 
 store s0 , sb , sc 
addi  sc ,  fp ,  5 
 store s1 , sb , sc 
li  x0 , 255 
addi  sc ,  fp ,  8 
 store x0 , sb , sc 
addi  sc ,  fp ,  7 
 store r0 , sb , sc 
loop_31:
beq  r0 ,  r0 , cont_32
beq r0, r0, end_33
cont_32: 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
 add  s2 , rv0 , r0 
addi  sc ,  fp ,  7 
 store s2 , sb , sc 
addi  sc ,  fp ,  8 
 load t0 , sb , sc  
beq  s2 ,  t0 ,  endloop_27 
li  sc , 240 
load t3 , gp , sc 
addi  s1 ,  s1 ,  1 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
 add  s2 , rv0 , r0 
addi  sc ,  fp ,  6 
 store s2 , sb , sc 
addi  s1 ,  s1 ,  1 
addi  s1 ,  s1 ,  1 
; "skip padding byte" 
li  s3 , 0 
loop_28:
bne  s3 ,  s2 , cont_29
beq r0, r0, end_30
cont_29: 
addi  sc ,  fp ,  7 
 load t1 , sb , sc  
add  t1 ,  t1 ,  s3 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
 add  t0 , rv0 , r0 
li  sc , 240 
load t3 , gp , sc 
add  a0 , t3 , r0 
add  a1 , t1 , r0 
add  a2 , t0 , r0 
call  set_image_byte 
addi  s1 ,  s1 ,  1 
addi  s3 ,  s3 ,  1 
beq r0, r0, loop_28
end_30: 
addi  s1 ,  s1 ,  1 
; "skip padding byte" 
beq r0, r0, loop_31
end_33: 
endloop_27:
addi  sp ,  sp ,  -9 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

load_titlepic:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  6 
    li  x0 , 76 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 117 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 109 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 112 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 32 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 99 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 111 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 117 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 110 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 116 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 58 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 32 
add  a0 , x0 , r0 
call  putchar 
li  a0 , 10 
call  putchar 
li  a0 , 0 
li  a1 , 4 
call  storage_read_u32_le 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
addi  sc ,  fp ,  0 
 store s1 , sb , sc 
add  a0 , s1 , r0 
call  print_uint 
li  x0 , 10 
li  sc , 0 
store x0 , r0 , sc 
addi  sc ,  fp ,  0 
 load s2 , sb , sc  
add  a0 , s2 , r0 
call  get_titlepic_table_pos 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  print_hex32 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  get_titlepic_header_pos 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  print_hex32 
addi  sc ,  fp ,  1 
 store s0 , sb , sc 
addi  sc ,  fp ,  2 
 store s1 , sb , sc 
li  x0 , 10 
li  sc , 0 
store x0 , r0 , sc 
li  x0 , 84 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 105 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 116 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 108 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 101 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 112 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 105 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 99 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 32 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 115 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 105 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 122 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 101 
add  a0 , x0 , r0 
call  putchar 
li  x0 , 58 
add  a0 , x0 , r0 
call  putchar 
li  a0 , 10 
call  putchar 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  get_image_size 
 add  x0 , rv0 , r0 
 add  x1 , rv1 , r0 
addi  sc ,  fp ,  3 
 store x0 , sb , sc 
addi  sc ,  fp ,  4 
 store x1 , sb , sc 
add  a0 , x0 , r0 
call  print_uint 
li  a0 , 120 
call  putchar 
add  a0 , x1 , r0 
call  print_uint 
li  x0 , 10 
li  sc , 0 
store x0 , r0 , sc 
addi  sc ,  fp ,  5 
 store r0 , sb , sc 
addi  sc ,  fp ,  5 
 load t0 , sb , sc  
addi  sc ,  fp ,  3 
 load t1 , sb , sc  
addi  sc ,  fp ,  1 
 load s0 , sb , sc  
addi  sc ,  fp ,  2 
 load s1 , sb , sc  
loop_34:
bne  t0 ,  t1 , cont_35
beq r0, r0, end_36
cont_35: 
addi  sc ,  fp ,  5 
 load t0 , sb , sc  
li  sc , 240 
store t0 , gp , sc 
; "Column index in global bank" 
muli  t2 ,  t0 ,  4 
addi  t2 ,  t2 ,  8 
add  t2 ,  t2 ,  s1 
add  a0 , s0 , r0 
add  a1 , t2 , r0 
call  storage_read_u32_le 
 add  s2 , rv0 , r0 
 add  s3 , rv1 , r0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
add  a2 , s2 , r0 
add  a3 , s3 , r0 
call  read_column 
addi  sc ,  fp ,  5 
 load t0 , sb , sc  
addi  sc ,  fp ,  3 
 load t1 , sb , sc  
addi  t0 ,  t0 ,  1 
addi  sc ,  fp ,  5 
 store t0 , sb , sc 
beq r0, r0, loop_34
end_36: 

addi  sp ,  sp ,  -6 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

rgb_to_rgb565:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
add  s2 , a2 , r0 
andi  s0 ,  s0 ,  248 
slli  s0 ,  s0 ,  8 
andi  s1 ,  s1 ,  252 
slli  s1 ,  s1 ,  3 
andi  s2 ,  s2 ,  248 
srli  s2 ,  s2 ,  3 
or  s0 ,  s0 ,  s1 
or  s0 ,  s0 ,  s2 
add  rv0 , s0 , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

graphics_init:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
slli  s0 ,  s0 ,  8 
or  s0 ,  s0 ,  s1 
li  sc , 16 
store s0 , r0 , sc 
li  s3 , 3 
li  sc , 6 
store s3 , r0 , sc 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

set_pixel:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
add  s2 , a2 , r0 
li  t0 , 160 
li  t1 , 100 
mul  t2 ,  t0 ,  t1 
addi  t2 ,  t2 ,  32 
mul  t3 ,  s1 ,  t0 
add  t3 ,  t3 ,  s0 
add  t3 ,  t3 ,  t2 
store s2 , r0 , t3 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

clear_screen:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
li  s1 , 0 
li  s2 , 100 
loop_40:
blt  s1 ,  s2 , cont_41
beq r0, r0, end_42
cont_41: 
li  s3 , 0 
li  t0 , 160 
loop_37:
blt  s3 ,  t0 , cont_38
beq r0, r0, end_39
cont_38: 
add  a0 , s3 , r0 
add  a1 , s1 , r0 
add  a2 , s0 , r0 
call  set_pixel 
addi  s3 ,  s3 ,  1 
beq r0, r0, loop_37
end_39: 
addi  s1 ,  s1 ,  1 
beq r0, r0, loop_40
end_42: 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 


read_rgb_565:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
 add  s2 , rv0 , r0 
addi  s1 ,  s1 ,  1 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
 add  s3 , rv0 , r0 
addi  s1 ,  s1 ,  1 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_at 
 add  t0 , rv0 , r0 
add  a0 , s2 , r0 
add  a1 , s3 , r0 
add  a2 , t0 , r0 
call  rgb_to_rgb565 
 add  s0 , rv0 , r0 
add  rv0 , s0 , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

load_palette:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  2 
    li  a0 , 0 
li  a1 , 8 
call  storage_read_u32_le 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
add  a0 , s0 , r0 
add  a1 , s1 , r0 
call  storage_read_u32_le 
 add  s0 , rv0 , r0 
 add  s1 , rv1 , r0 
addi  sc ,  fp ,  0 
 store s0 , sb , sc 
addi  sc ,  fp ,  1 
 store s1 , sb , sc 
li  s2 , 256 
li  s3 , 0 
loop_43:
bne  s3 ,  s2 , cont_44
beq r0, r0, end_45
cont_44: 
muli  t1 ,  s3 ,  3 
add  t1 ,  t1 ,  s1 
add  a0 , s0 , r0 
add  a1 , t1 , r0 
call  read_rgb_565 
 add  t2 , rv0 , r0 
li  sc , 4 
store t2 , sc , s3 
addi  s3 ,  s3 ,  1 
beq r0, r0, loop_43
end_45: 

addi  sp ,  sp ,  -2 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

resolve_titlepic_colors:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    li  s0 , 0 
li  s1 , 63999 
li  s3 , 3 
loop_46:
bne  s0 ,  s1 , cont_47
beq r0, r0, end_48
cont_47: 
li  t0 , 4 
load s2 , s3 , s0  
load s2 , t0 , s2  
store s2 , s3 , s0 
addi  s0 ,  s0 ,  1 
beq r0, r0, loop_46
end_48: 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

get_titlepic_pixel:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    add  s0 , a0 , r0 
add  s1 , a1 , r0 
li  t0 , 200 
mul  t2 ,  s0 ,  t0 
add  t2 ,  t2 ,  s1 
li  sc , 3 
load s2 , sc , t2  
add  rv0 , s2 , r0 
add  rv1 , r0 , r0 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra 

main:
store ra , sb , sp 
 addi  sp ,  sp ,  1 
store fp , sb , sp 
 addi  sp ,  sp ,  1 
store s0 , sb , sp 
 addi  sp ,  sp ,  1 
store s1 , sb , sp 
 addi  sp ,  sp ,  1 
store s2 , sb , sp 
 addi  sp ,  sp ,  1 
store s3 , sb , sp 
 addi  sp ,  sp ,  1 
add  fp , sp , r0 
addi  sp ,  sp ,  0 
    call  iwad 
call  load_titlepic 
call  load_palette 
call  resolve_titlepic_colors 
li  a0 , 160 
li  a1 , 100 
call  graphics_init 
loop_55:
beq  r0 ,  r0 , cont_56
beq r0, r0, end_57
cont_56: 
li  a0 , 65535 
call  clear_screen 
li  s0 , 0 
li  s1 , 100 
li  s2 , 0 
li  s3 , 160 
loop_52:
blt  s0 ,  s1 , cont_53
beq r0, r0, end_54
cont_53: 
li  s2 , 0 
loop_49:
blt  s2 ,  s3 , cont_50
beq r0, r0, end_51
cont_50: 
muli  x0 ,  s2 ,  2 
muli  x1 ,  s0 ,  2 
add  a0 , x0 , r0 
add  a1 , x1 , r0 
call  get_titlepic_pixel 
 add  t0 , rv0 , r0 
add  a0 , s2 , r0 
add  a1 , s0 , r0 
add  a2 , t0 , r0 
call  set_pixel 
addi  s2 ,  s2 ,  1 
beq r0, r0, loop_49
end_51: 
addi  s0 ,  s0 ,  1 
beq r0, r0, loop_52
end_54: 
call  graphics_flush 
beq r0, r0, loop_55
end_57: 

addi  sp ,  sp ,  -0 
addi  sp ,  sp ,  -1 
 load s3 , sb , sp  
addi  sp ,  sp ,  -1 
 load s2 , sb , sp  
addi  sp ,  sp ,  -1 
 load s1 , sb , sp  
addi  sp ,  sp ,  -1 
 load s0 , sb , sp  
addi  sp ,  sp ,  -1 
 load fp , sb , sp  
addi  sp ,  sp ,  -1 
 load ra , sb , sp  
jalr rab, rab, ra