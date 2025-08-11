R5 == 0x48 - Register equals value
R0 != R1 - Register not equals register
R13 < 0x10 - Less than
R13 > 0x10 - Greater than
R13 <= 0x10 - Less than or equal
R13 >= 0x10 - Greater than or equal
[0x1000] == 42 - Memory at address equals value
[R5+R6] != 0 - Memory at computed address
changed(R5) - Register value changed since last instruction
R5 & 0xFF == 0x48 - Bitwise AND
R0 + R1 > 0x100 - Arithmetic expressions
PC == 0x100 && R0 != 0 - AND multiple conditions
R5 == 0x48 && R6 == 0x65 - Chain conditions
RA == 0 && PC > 0x2000 - Complex combinations