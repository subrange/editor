// Full Forth Implementation
// A complete Forth interpreter with dictionary, control structures, and word definitions

#include <stdio.h>

// Stack and memory configuration
#define STACK_SIZE 256
#define RETURN_STACK_SIZE 256
#define DICT_SIZE 1024
#define INPUT_BUFFER_SIZE 256
#define MAX_WORD_LENGTH 32

// Forth VM state
int data_stack[STACK_SIZE];
int return_stack[RETURN_STACK_SIZE];
int sp = 0;  // Data stack pointer
int rsp = 0; // Return stack pointer

// Dictionary structures
typedef void (*PrimitiveFn)(void);

typedef struct {
    char name[MAX_WORD_LENGTH];
    int is_immediate;
    int is_primitive;
    PrimitiveFn primitive_fn;
    int code_addr; // For user-defined words
} Word;

Word dictionary[DICT_SIZE];
int dict_count = 0;

// Memory for compiled code
int memory[4096];
int here = 0; // Next free memory location

// Interpreter state
int compile_mode = 0;
int ip = 0; // Instruction pointer

// Input buffer
char input_buffer[INPUT_BUFFER_SIZE];
char token_buffer[MAX_WORD_LENGTH];

// Error handling
void error(const char* msg) {
    puts("ERROR: ");
    puts(msg);
}

// Stack operations
void push(int val) {
    if (sp >= STACK_SIZE) {
        error("Stack overflow");
        return;
    }
    data_stack[sp++] = val;
}

int pop() {
    if (sp <= 0) {
        error("Stack underflow");
        return 0;
    }
    return data_stack[--sp];
}

void rpush(int val) {
    if (rsp >= RETURN_STACK_SIZE) {
        error("Return stack overflow");
        return;
    }
    return_stack[rsp++] = val;
}

int rpop() {
    if (rsp <= 0) {
        error("Return stack underflow");
        return 0;
    }
    return return_stack[--rsp];
}

// Number printing
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

// String utilities
int str_eq(const char* a, const char* b) {
    int i = 0;
    while (a[i] && b[i]) {
        if (a[i] != b[i]) return 0;
        i++;
    }
    return a[i] == b[i];
}

void str_copy(char* dst, const char* src) {
    int i = 0;
    while (src[i] && i < MAX_WORD_LENGTH - 1) {
        dst[i] = src[i];
        i++;
    }
    dst[i] = 0;
}

int str_len(const char* s) {
    int i = 0;
    while (s[i]) i++;
    return i;
}

// Dictionary lookup
Word* find_word(const char* name) {
    for (int i = dict_count - 1; i >= 0; i--) {
        if (str_eq(dictionary[i].name, name)) {
            return &dictionary[i];
        }
    }
    return 0;
}

// Primitive words implementation
void prim_plus() {
    int b = pop();
    int a = pop();
    push(a + b);
}

void prim_minus() {
    int b = pop();
    int a = pop();
    push(a - b);
}

void prim_mul() {
    int b = pop();
    int a = pop();
    push(a * b);
}

void prim_div() {
    int b = pop();
    int a = pop();
    if (b == 0) {
        error("Division by zero");
        push(a);
        return;
    }
    push(a / b);
}

void prim_mod() {
    int b = pop();
    int a = pop();
    if (b == 0) {
        error("Division by zero");
        push(a);
        return;
    }
    push(a % b);
}

void prim_eq() {
    int b = pop();
    int a = pop();
    push(a == b ? -1 : 0);
}

void prim_ne() {
    int b = pop();
    int a = pop();
    push(a != b ? -1 : 0);
}

void prim_lt() {
    int b = pop();
    int a = pop();
    push(a < b ? -1 : 0);
}

void prim_gt() {
    int b = pop();
    int a = pop();
    push(a > b ? -1 : 0);
}

void prim_le() {
    int b = pop();
    int a = pop();
    push(a <= b ? -1 : 0);
}

void prim_ge() {
    int b = pop();
    int a = pop();
    push(a >= b ? -1 : 0);
}

void prim_and() {
    int b = pop();
    int a = pop();
    push(a & b);
}

void prim_or() {
    int b = pop();
    int a = pop();
    push(a | b);
}

void prim_xor() {
    int b = pop();
    int a = pop();
    push(a ^ b);
}

void prim_not() {
    push(~pop());
}

void prim_dup() {
    if (sp > 0) {
        int val = data_stack[sp - 1];
        push(val);
    }
}

void prim_drop() {
    pop();
}

void prim_swap() {
    if (sp >= 2) {
        int temp = data_stack[sp - 1];
        data_stack[sp - 1] = data_stack[sp - 2];
        data_stack[sp - 2] = temp;
    }
}

void prim_over() {
    if (sp >= 2) {
        push(data_stack[sp - 2]);
    }
}

void prim_rot() {
    if (sp >= 3) {
        int c = pop();
        int b = pop();
        int a = pop();
        push(b);
        push(c);
        push(a);
    }
}

void prim_dot() {
    if (sp > 0) {
        print_number(pop());
        putchar(' ');
    } else {
        error("Stack empty");
    }
}

void prim_cr() {
    putchar('\n');
}

void prim_space() {
    putchar(' ');
}

void prim_emit() {
    if (sp > 0) {
        putchar(pop());
    }
}

void prim_dots() {
    puts("Stack: ");
    if (sp == 0) {
        puts("(empty)");
    } else {
        for (int i = 0; i < sp; i++) {
            print_number(data_stack[i]);
            putchar(' ');
        }
        putchar('\n');
    }
}

void prim_words() {
    puts("Dictionary:");
    for (int i = 0; i < dict_count; i++) {
        puts(dictionary[i].name);
        putchar(' ');
        if ((i + 1) % 8 == 0) putchar('\n');
    }
    putchar('\n');
}

void prim_colon() {
    compile_mode = 1;
}

void prim_semicolon() {
    memory[here++] = -1; // Return marker
    compile_mode = 0;
}

void prim_to_r() {
    rpush(pop());
}

void prim_from_r() {
    push(rpop());
}

void prim_r_at() {
    if (rsp > 0) {
        push(return_stack[rsp - 1]);
    }
}

void prim_if() {
    memory[here] = -2; // IF marker
    push(here);
    here++;
}

void prim_else() {
    int if_addr = pop();
    memory[here] = -3; // ELSE marker
    push(here);
    here++;
    memory[if_addr] = here; // Patch IF to jump here
}

void prim_then() {
    int addr = pop();
    memory[addr] = here; // Patch jump address
}

void prim_do() {
    memory[here] = -4; // DO marker
    push(here);
}

void prim_loop() {
    int do_addr = pop();
    memory[here++] = -5; // LOOP marker
    memory[here++] = do_addr;
}

void prim_i() {
    if (rsp >= 2) {
        push(return_stack[rsp - 2]); // Loop index
    }
}

void prim_j() {
    if (rsp >= 4) {
        push(return_stack[rsp - 4]); // Outer loop index
    }
}

void prim_bye() {
    puts("Goodbye!");
    // In real implementation, would exit
    // For now, just set a flag
    compile_mode = -1; // Use as exit flag
}

// Add primitive to dictionary
void add_primitive(const char* name, void (*fn)(), int immediate) {
    if (dict_count >= DICT_SIZE) {
        error("Dictionary full");
        return;
    }
    
    str_copy(dictionary[dict_count].name, name);
    dictionary[dict_count].is_primitive = 1;
    dictionary[dict_count].is_immediate = immediate;
    dictionary[dict_count].primitive_fn = fn;
    dictionary[dict_count].code_addr = -1;
    dict_count++;
}

// Initialize dictionary with primitives
void init_dictionary() {
    // Arithmetic
    add_primitive("+", prim_plus, 0);
    add_primitive("-", prim_minus, 0);
    add_primitive("*", prim_mul, 0);
    add_primitive("/", prim_div, 0);
    add_primitive("MOD", prim_mod, 0);
    
    // Comparison
    add_primitive("=", prim_eq, 0);
    add_primitive("<>", prim_ne, 0);
    add_primitive("<", prim_lt, 0);
    add_primitive(">", prim_gt, 0);
    add_primitive("<=", prim_le, 0);
    add_primitive(">=", prim_ge, 0);
    
    // Logical
    add_primitive("AND", prim_and, 0);
    add_primitive("OR", prim_or, 0);
    add_primitive("XOR", prim_xor, 0);
    add_primitive("NOT", prim_not, 0);
    
    // Stack manipulation
    add_primitive("DUP", prim_dup, 0);
    add_primitive("DROP", prim_drop, 0);
    add_primitive("SWAP", prim_swap, 0);
    add_primitive("OVER", prim_over, 0);
    add_primitive("ROT", prim_rot, 0);
    
    // Return stack
    add_primitive(">R", prim_to_r, 0);
    add_primitive("R>", prim_from_r, 0);
    add_primitive("R@", prim_r_at, 0);
    
    // I/O
    add_primitive(".", prim_dot, 0);
    add_primitive("CR", prim_cr, 0);
    add_primitive("SPACE", prim_space, 0);
    add_primitive("EMIT", prim_emit, 0);
    add_primitive(".S", prim_dots, 0);
    
    // Dictionary
    add_primitive("WORDS", prim_words, 0);
    
    // Compilation
    add_primitive(":", prim_colon, 0);
    add_primitive(";", prim_semicolon, 1); // Immediate
    
    // Control structures
    add_primitive("IF", prim_if, 1);
    add_primitive("ELSE", prim_else, 1);
    add_primitive("THEN", prim_then, 1);
    add_primitive("DO", prim_do, 1);
    add_primitive("LOOP", prim_loop, 1);
    add_primitive("I", prim_i, 0);
    add_primitive("J", prim_j, 0);
    
    // System
    add_primitive("BYE", prim_bye, 0);
}

// Parse number from string
int parse_number(const char* str, int* result) {
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

// Get next token from input
int get_token(char* token) {
    static int pos = 0;
    int i = 0;
    
    // Skip whitespace
    while (input_buffer[pos] && 
           (input_buffer[pos] == ' ' || 
            input_buffer[pos] == '\t' || 
            input_buffer[pos] == '\n')) {
        pos++;
    }
    
    // Check for end of input
    if (!input_buffer[pos]) {
        pos = 0;
        return 0;
    }
    
    // Copy token
    while (input_buffer[pos] && 
           input_buffer[pos] != ' ' && 
           input_buffer[pos] != '\t' && 
           input_buffer[pos] != '\n' &&
           i < MAX_WORD_LENGTH - 1) {
        token[i++] = input_buffer[pos++];
    }
    token[i] = 0;
    
    return i > 0;
}

// Execute a word (primitive or user-defined)
void execute_word(Word* word) {
    if (word->is_primitive) {
        word->primitive_fn();
    } else {
        // Execute user-defined word
        rpush(ip); // Save current IP
        ip = word->code_addr;
        
        while (memory[ip] != -1) { // -1 is return marker
            if (memory[ip] >= 0 && memory[ip] < dict_count) {
                // It's a word reference
                execute_word(&dictionary[memory[ip]]);
            } else if (memory[ip] == -2) { // IF
                ip++;
                int cond = pop();
                if (!cond) {
                    ip = memory[ip]; // Jump to ELSE or THEN
                    continue;
                }
            } else if (memory[ip] == -3) { // ELSE
                ip++;
                ip = memory[ip]; // Jump to THEN
                continue;
            } else if (memory[ip] == -4) { // DO
                int limit = pop();
                int start = pop();
                rpush(limit);
                rpush(start);
            } else if (memory[ip] == -5) { // LOOP
                ip++;
                int loop_addr = memory[ip];
                int index = rpop();
                int limit = rpop();
                index++;
                if (index < limit) {
                    rpush(limit);
                    rpush(index);
                    ip = loop_addr;
                    continue;
                }
            } else {
                // It's a literal number
                push(memory[ip]);
            }
            ip++;
        }
        
        ip = rpop(); // Restore IP
    }
}

// Process a token
void process_token(char* token) {
    // Check if it's a word in dictionary
    Word* word = find_word(token);
    
    if (word) {
        if (compile_mode && !word->is_immediate) {
            // Compile the word reference
            int word_index = word - dictionary;
            memory[here++] = word_index;
        } else {
            // Execute the word
            execute_word(word);
        }
    } else {
        // Try to parse as number
        int num;
        if (parse_number(token, &num)) {
            if (compile_mode) {
                memory[here++] = num;
            } else {
                push(num);
            }
        } else if (compile_mode && token[0] == ':') {
            // Skip, already handled
        } else if (compile_mode) {
            // Creating new word definition
            if (dict_count >= DICT_SIZE) {
                error("Dictionary full");
                return;
            }
            
            str_copy(dictionary[dict_count].name, token);
            dictionary[dict_count].is_primitive = 0;
            dictionary[dict_count].is_immediate = 0;
            dictionary[dict_count].primitive_fn = 0;
            dictionary[dict_count].code_addr = here;
            dict_count++;
            
            // Now compile the definition
            compile_mode = 2; // Special mode for compiling body
        } else {
            puts("Unknown word: ");
            puts(token);
            putchar('\n');
        }
    }
}

// Read line from input
void read_line() {
    int i = 0;
    int ch;
    
    while (i < INPUT_BUFFER_SIZE - 1) {
        ch = getchar();
        
        if (ch == '\n') {
            input_buffer[i] = 0;
            return;
        }
        
        putchar(ch); // Echo
        input_buffer[i++] = ch;
    }
    input_buffer[i] = 0;
}

int main() {
    puts("Forth Interpreter");
    puts("================");
    puts("Type 'WORDS' to see available words");
    puts("Type 'BYE' to exit");
    puts("");
    
    // Initialize dictionary
    init_dictionary();
    
    // Main interpreter loop
    while (compile_mode != -1) { // -1 is exit flag
        if (!compile_mode) {
            puts(" ok");
            putchar('>');
            putchar(' ');
        }
        
        read_line();
        
        // Process all tokens in the line
        while (get_token(token_buffer)) {
            if (compile_mode == 2) {
                // We just got the name, now compile the body
                compile_mode = 1;
                continue;
            }
            process_token(token_buffer);
        }
    }
    
    return 0;
}