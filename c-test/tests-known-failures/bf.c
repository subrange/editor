void putchar(int c);
int puts(char *str);

/* Execute a Brainfuck program from a NUL-terminated string.
   Tape cells are 16-bit (unsigned short). Data pointer and values wrap. */
void bf_exec(char *code) {
    /* constants (no preprocessor): */
    int TAPE_SIZE = 1024;

    /* tape and state */
    unsigned short tape[1024];
    int i = 0;
    int dp = 0;

    /* init tape */
    for (i = 0; i < TAPE_SIZE; i = i + 1) tape[i] = 0;

    /* compute program length (avoid strlen) */
    int len = 0;
    while (code[len] != 0) len = len + 1;

    /* interpreter */
    i = 0;
    while (i < len) {
        char op = code[i];

        if (op == '>') {
            dp = dp + 1;
            if (dp >= TAPE_SIZE) dp = 0;
        } else if (op == '<') {
            dp = dp - 1;
            if (dp < 0) dp = TAPE_SIZE - 1;
        } else if (op == '+') {
            tape[dp] = (unsigned short)(tape[dp] + 1);
        } else if (op == '-') {
            tape[dp] = (unsigned short)(tape[dp] - 1);
        } else if (op == '.') {
            putchar((int)(tape[dp] & 0xFF));
        } else if (op == ',') {
            tape[dp] = 0;
        } else if (op == '[') {
            if (tape[dp] == 0) {

                int depth = 1;
                int j = i + 1;
                while (j < len && depth > 0) {
                    char c = code[j];
                    if (c == '[') depth = depth + 1;
                    else if (c == ']') depth = depth - 1;
                    j = j + 1;
                }

                i = j - 1;
            }
        } else if (op == ']') {
            if (tape[dp] != 0) {
                int depth = 1;
                int j = i - 1;
                while (j >= 0 && depth > 0) {
                    char c = code[j];
                    if (c == ']') depth = depth + 1;
                    else if (c == '[') depth = depth - 1;
                    j = j - 1;
                }
                i = j + 1;
            }
        } else {

        }

        i = i + 1;
    }
}

void main() {
    char *code = "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>>>+++++++++++.>++.<<+++++++++++++++.>.+++.------.--------.>>>+++++++++++.>+.>.";

    puts("=== Brainfuck Hello World ===");
    puts("This is a Brainfuck interpreter written in C. Running on the Ripple VM compiled to Brainfuck.");

    bf_exec(code);
}