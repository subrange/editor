// Simple Forth Implementation without function pointers or switch
// Uses if-else chains for word dispatch

#include <stdio.h>

// Stack configuration
#define STACK_SIZE 100
#define MAX_WORDS 100
#define MAX_WORD_LEN 32
#define CODE_SIZE 1000

// Data stack
int stack[STACK_SIZE];
int sp = 0;

// Return stack for loops
int rstack[STACK_SIZE];
int rsp = 0;

// Dictionary entry
struct Word {
    char name[MAX_WORD_LEN];
    int is_primitive;  // 1 for built-in, 0 for user-defined
    int code_start;    // Index into code array for user words
    int immediate;     // 1 for immediate words
};

struct Word dictionary[MAX_WORDS];
int dict_count = 0;

// Code storage for user-defined words
int code[CODE_SIZE];
int here = 0;  // Next free code location

// Interpreter state
int compile_mode = 0;
int ip = 0;  // Instruction pointer

// Input buffer
char input_line[256];
char current_word[MAX_WORD_LEN];

// Stack operations
void push(int val) {
    if (sp < STACK_SIZE) {
        stack[sp++] = val;
    } else {
        puts("Stack overflow!");
    }
}

int pop() {
    if (sp > 0) {
        return stack[--sp];
    }
    puts("Stack underflow!");
    return 0;
}

void rpush(int val) {
    if (rsp < STACK_SIZE) {
        rstack[rsp++] = val;
    }
}

int rpop() {
    if (rsp > 0) {
        return rstack[--rsp];
    }
    return 0;
}

// String utilities
int str_eq(char* a, char* b) {
    int i = 0;
    while (a[i] && b[i]) {
        if (a[i] != b[i]) return 0;
        i++;
    }
    return a[i] == b[i];
}

void str_copy(char* dst, char* src) {
    int i = 0;
    while (src[i] && i < MAX_WORD_LEN - 1) {
        dst[i] = src[i];
        i++;
    }
    dst[i] = 0;
}

int str_len(char* s) {
    int i = 0;
    while (s[i]) i++;
    return i;
}

// Number utilities
void print_number(int n) {
    if (n == 0) {
        putchar('0');
        return;
    }
    
    if (n < 0) {
        putchar('-');
        n = -n;
    }
    
    char digits[12];
    int i = 0;
    while (n > 0) {
        digits[i++] = '0' + (n % 10);
        n = n / 10;
    }
    
    while (i > 0) {
        putchar(digits[--i]);
    }
}

int parse_number(char* str, int* result) {
    int val = 0;
    int sign = 1;
    int i = 0;
    
    if (str[0] == '-') {
        sign = -1;
        i = 1;
    }
    
    if (!str[i]) return 0;
    
    while (str[i]) {
        if (str[i] < '0' || str[i] > '9') {
            return 0;
        }
        val = val * 10 + (str[i] - '0');
        i++;
    }
    
    *result = val * sign;
    return 1;
}

// Find word in dictionary
int find_word(char* name) {
    for (int i = dict_count - 1; i >= 0; i--) {
        if (str_eq(dictionary[i].name, name)) {
            return i;
        }
    }
    return -1;
}

// Add word to dictionary
void add_word(char* name, int is_prim, int code_idx, int immed) {
    if (dict_count < MAX_WORDS) {
        str_copy(dictionary[dict_count].name, name);
        dictionary[dict_count].is_primitive = is_prim;
        dictionary[dict_count].code_start = code_idx;
        dictionary[dict_count].immediate = immed;
        dict_count++;
    }
}

// Execute primitive word by name
void exec_primitive(char* name) {
    // Arithmetic
    if (str_eq(name, "+")) {
        int b = pop();
        int a = pop();
        push(a + b);
    }
    else if (str_eq(name, "-")) {
        int b = pop();
        int a = pop();
        push(a - b);
    }
    else if (str_eq(name, "*")) {
        int b = pop();
        int a = pop();
        push(a * b);
    }
    else if (str_eq(name, "/")) {
        int b = pop();
        int a = pop();
        if (b != 0) {
            push(a / b);
        } else {
            puts("Division by zero!");
            push(a);
        }
    }
    else if (str_eq(name, "MOD")) {
        int b = pop();
        int a = pop();
        if (b != 0) {
            push(a % b);
        } else {
            puts("Division by zero!");
            push(a);
        }
    }
    // Comparison
    else if (str_eq(name, "=")) {
        int b = pop();
        int a = pop();
        push(a == b ? -1 : 0);
    }
    else if (str_eq(name, "<")) {
        int b = pop();
        int a = pop();
        push(a < b ? -1 : 0);
    }
    else if (str_eq(name, ">")) {
        int b = pop();
        int a = pop();
        push(a > b ? -1 : 0);
    }
    // Stack manipulation
    else if (str_eq(name, "DUP")) {
        if (sp > 0) {
            push(stack[sp - 1]);
        }
    }
    else if (str_eq(name, "DROP")) {
        pop();
    }
    else if (str_eq(name, "SWAP")) {
        if (sp >= 2) {
            int temp = stack[sp - 1];
            stack[sp - 1] = stack[sp - 2];
            stack[sp - 2] = temp;
        }
    }
    else if (str_eq(name, "OVER")) {
        if (sp >= 2) {
            push(stack[sp - 2]);
        }
    }
    else if (str_eq(name, "ROT")) {
        if (sp >= 3) {
            int c = pop();
            int b = pop();
            int a = pop();
            push(b);
            push(c);
            push(a);
        }
    }
    // I/O
    else if (str_eq(name, ".")) {
        if (sp > 0) {
            print_number(pop());
            putchar(' ');
        } else {
            puts("Stack empty!");
        }
    }
    else if (str_eq(name, "CR")) {
        putchar('\n');
    }
    else if (str_eq(name, "EMIT")) {
        if (sp > 0) {
            putchar(pop());
        }
    }
    else if (str_eq(name, ".S")) {
        puts("Stack:");
        if (sp == 0) {
            puts("  (empty)");
        } else {
            for (int i = 0; i < sp; i++) {
                putchar(' ');
                putchar(' ');
                print_number(stack[i]);
                putchar('\n');
            }
        }
    }
    else if (str_eq(name, "WORDS")) {
        puts("Dictionary:");
        for (int i = 0; i < dict_count; i++) {
            puts(dictionary[i].name);
            putchar(' ');
        }
        putchar('\n');
    }
    // Return stack
    else if (str_eq(name, ">R")) {
        rpush(pop());
    }
    else if (str_eq(name, "R>")) {
        push(rpop());
    }
    else if (str_eq(name, "R@")) {
        if (rsp > 0) {
            push(rstack[rsp - 1]);
        }
    }
    else if (str_eq(name, "I")) {
        if (rsp >= 2) {
            push(rstack[rsp - 2]);
        }
    }
    // Control
    else if (str_eq(name, "BYE")) {
        puts("Goodbye!");
        compile_mode = -999;  // Exit flag
    }
}

// Execute a word (by index)
void execute(int word_idx) {
    if (word_idx < 0 || word_idx >= dict_count) return;
    
    if (dictionary[word_idx].is_primitive) {
        exec_primitive(dictionary[word_idx].name);
    } else {
        // Execute user-defined word
        rpush(ip);  // Save current IP
        ip = dictionary[word_idx].code_start;
        
        while (code[ip] != -1) {  // -1 marks end of word
            if (code[ip] >= 0 && code[ip] < dict_count) {
                // It's a word reference
                execute(code[ip]);
            } else if (code[ip] == -2) {
                // IF: pop condition and skip if false
                ip++;
                if (!pop()) {
                    ip = code[ip];  // Jump to ELSE/THEN
                    continue;
                }
            } else if (code[ip] == -3) {
                // ELSE: jump to THEN
                ip++;
                ip = code[ip];
                continue;
            } else if (code[ip] == -4) {
                // DO: setup loop
                int limit = pop();
                int start = pop();
                rpush(limit);
                rpush(start);
            } else if (code[ip] == -5) {
                // LOOP: increment and check
                ip++;
                int loop_addr = code[ip];
                int index = rpop();
                int limit = rpop();
                index++;
                if (index < limit) {
                    rpush(limit);
                    rpush(index);
                    ip = loop_addr;
                    continue;
                }
            } else if (code[ip] >= 10000) {
                // It's a literal number (offset by 10000)
                push(code[ip] - 10000);
            } else if (code[ip] <= -10000) {
                // Negative literal
                push(code[ip] + 10000);
            }
            ip++;
        }
        
        ip = rpop();  // Restore IP
    }
}

// Initialize built-in words
void init_dict() {
    // Let's try a different approach - build strings directly in dictionary
    
    // + 
    dictionary[dict_count].name[0] = '+';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // -
    dictionary[dict_count].name[0] = '-';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // *
    dictionary[dict_count].name[0] = '*';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // /
    dictionary[dict_count].name[0] = '/';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // MOD
    dictionary[dict_count].name[0] = 'M';
    dictionary[dict_count].name[1] = 'O';
    dictionary[dict_count].name[2] = 'D';
    dictionary[dict_count].name[3] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // =
    dictionary[dict_count].name[0] = '=';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // <
    dictionary[dict_count].name[0] = '<';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // >
    dictionary[dict_count].name[0] = '>';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // DUP
    dictionary[dict_count].name[0] = 'D';
    dictionary[dict_count].name[1] = 'U';
    dictionary[dict_count].name[2] = 'P';
    dictionary[dict_count].name[3] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // DROP
    dictionary[dict_count].name[0] = 'D';
    dictionary[dict_count].name[1] = 'R';
    dictionary[dict_count].name[2] = 'O';
    dictionary[dict_count].name[3] = 'P';
    dictionary[dict_count].name[4] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // SWAP
    dictionary[dict_count].name[0] = 'S';
    dictionary[dict_count].name[1] = 'W';
    dictionary[dict_count].name[2] = 'A';
    dictionary[dict_count].name[3] = 'P';
    dictionary[dict_count].name[4] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // OVER
    dictionary[dict_count].name[0] = 'O';
    dictionary[dict_count].name[1] = 'V';
    dictionary[dict_count].name[2] = 'E';
    dictionary[dict_count].name[3] = 'R';
    dictionary[dict_count].name[4] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // ROT
    dictionary[dict_count].name[0] = 'R';
    dictionary[dict_count].name[1] = 'O';
    dictionary[dict_count].name[2] = 'T';
    dictionary[dict_count].name[3] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // .
    dictionary[dict_count].name[0] = '.';
    dictionary[dict_count].name[1] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // CR
    dictionary[dict_count].name[0] = 'C';
    dictionary[dict_count].name[1] = 'R';
    dictionary[dict_count].name[2] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // EMIT
    dictionary[dict_count].name[0] = 'E';
    dictionary[dict_count].name[1] = 'M';
    dictionary[dict_count].name[2] = 'I';
    dictionary[dict_count].name[3] = 'T';
    dictionary[dict_count].name[4] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // .S
    dictionary[dict_count].name[0] = '.';
    dictionary[dict_count].name[1] = 'S';
    dictionary[dict_count].name[2] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // WORDS
    dictionary[dict_count].name[0] = 'W';
    dictionary[dict_count].name[1] = 'O';
    dictionary[dict_count].name[2] = 'R';
    dictionary[dict_count].name[3] = 'D';
    dictionary[dict_count].name[4] = 'S';
    dictionary[dict_count].name[5] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
    
    // BYE
    dictionary[dict_count].name[0] = 'B';
    dictionary[dict_count].name[1] = 'Y';
    dictionary[dict_count].name[2] = 'E';
    dictionary[dict_count].name[3] = 0;
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].code_start = 0;
    dictionary[dict_count].immediate = 0;
    dict_count++;
}

// Get next word from input line
int get_word(char* word, int* pos) {
    int i = 0;
    
    // Skip whitespace
    while (input_line[*pos] == ' ' || input_line[*pos] == '\t') {
        (*pos)++;
    }
    
    // Check end of line
    if (input_line[*pos] == 0 || input_line[*pos] == '\n') {
        return 0;
    }
    
    // Copy word
    while (input_line[*pos] && input_line[*pos] != ' ' && 
           input_line[*pos] != '\t' && input_line[*pos] != '\n' && 
           i < MAX_WORD_LEN - 1) {
        word[i++] = input_line[(*pos)++];
    }
    word[i] = 0;
    
    return i > 0;
}

// Process input line
void process_line() {
    int pos = 0;
    
    while (get_word(current_word, &pos)) {
        // Check for colon definition
        if (str_eq(current_word, ":")) {
            compile_mode = 1;
            // Get word name
            if (get_word(current_word, &pos)) {
                add_word(current_word, 0, here, 0);
            }
            continue;
        }
        
        if (str_eq(current_word, ";")) {
            if (compile_mode) {
                code[here++] = -1;  // End marker
                compile_mode = 0;
            }
            continue;
        }
        
        // Look up word
        int word_idx = find_word(current_word);
        
        if (word_idx >= 0) {
            if (compile_mode && !dictionary[word_idx].immediate) {
                // Compile word reference
                code[here++] = word_idx;
            } else {
                // Execute word
                execute(word_idx);
            }
        } else {
            // Try to parse as number
            int num;
            if (parse_number(current_word, &num)) {
                if (compile_mode) {
                    if (num >= 0) {
                        code[here++] = num + 10000;  // Positive literal
                    } else {
                        code[here++] = num - 10000;  // Negative literal
                    }
                } else {
                    push(num);
                }
            } else {
                puts("Unknown word: ");
                puts(current_word);
                putchar('\n');
            }
        }
    }
}

int main() {
    putchar('S');
    putchar('t');
    putchar('a');
    putchar('r');
    putchar('t');
    putchar('\n');
    
    puts("Simple Forth Interpreter");
    puts("========================");
    puts("Basic words: + - * / = < > DUP DROP SWAP . CR .S WORDS BYE");
    puts("Define words with : name ... ;");
    puts("");
    
    putchar('I');
    putchar('n');
    putchar('i');
    putchar('t');
    putchar('\n');
    
    init_dict();
    
    putchar('D');
    putchar('o');
    putchar('n');
    putchar('e');
    putchar('\n');
    
    // Main loop
    while (compile_mode != -999) {
        if (!compile_mode) {
            putchar('>');
            putchar(' ');
        }
        
        // Read line
        int i = 0;
        int ch;
        while (i < 255) {
            ch = getchar();
            if (ch == '\n') {
                input_line[i] = 0;
                break;
            }
            putchar(ch);  // Echo the character
            input_line[i++] = ch;
        }
        input_line[i] = 0;
        
        process_line();
        
        if (!compile_mode && compile_mode != -999) {
            puts(" ok");
        }
    }
    
    return 0;
}