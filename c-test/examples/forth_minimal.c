// Minimal Forth without structs containing arrays
// Uses parallel arrays to avoid compiler limitations
#include <stdio.h>
#include <string.h>

// Configuration
#define STACK_SIZE 100
#define MAX_WORDS 50
#define CODE_SIZE 500

// Data stack
int stack[STACK_SIZE];
int sp = 0;

// Dictionary - using parallel arrays instead of struct with array
char dict_names[1600];  // 50 * 32 = 1600 - Flattened array for names
int dict_is_prim[MAX_WORDS];
int dict_code_start[MAX_WORDS];
int dict_count = 0;

// Code storage
int code[CODE_SIZE];
int here = 0;

// State
int compile_mode = 0;
int running = 1;

// Global word buffer to avoid passing local arrays
char global_word[32];

// Stack operations
void push(int val) {
    if (sp < STACK_SIZE) {
        stack[sp++] = val;
    }
}

int pop() {
    if (sp > 0) {
        return stack[--sp];
    }
    return 0;
}

// Get name pointer for dictionary entry
char* get_dict_name(int idx) {
    return &dict_names[idx * 32];
}

// String compare - wrapper for strcmp that returns 1 for equal
int str_eq(char* a, char* b) {
    return strcmp(a, b) == 0;
}

// String copy with length limit
void str_copy(char* dst, char* src) {
    strncpy(dst, src, 31);
    dst[31] = 0;  // Ensure null termination
}

// Print number
void print_num(int n) {
    if (n == 0) {
        putchar('0');
        return;
    }
    if (n < 0) {
        putchar('-');
        n = -n;
    }
    char buf[12];
    int i = 0;
    while (n > 0) {
        buf[i++] = '0' + (n % 10);
        n = n / 10;
    }
    while (i > 0) {
        putchar(buf[--i]);
    }
}

// Parse number
int parse_num(char* s, int* result) {
    int val = 0;
    int i = 0;
    int sign = 1;
    
    if (s[0] == '-') {
        sign = -1;
        i = 1;
    }
    
    while (s[i]) {
        if (s[i] < '0' || s[i] > '9') return 0;
        val = val * 10 + (s[i] - '0');
        i++;
    }
    
    *result = val * sign;
    return 1;
}

// Find word
int find_word(char* name) {
    for (int i = 0; i < dict_count; i++) {
        if (str_eq(get_dict_name(i), name)) {
            return i;
        }
    }
    return -1;
}

// Add word to dictionary (not used for user words anymore)
void add_word(char* name, int is_prim, int code_start) {
    if (dict_count < MAX_WORDS) {
        char* dst = get_dict_name(dict_count);
        str_copy(dst, name);
        
        dict_is_prim[dict_count] = is_prim;
        dict_code_start[dict_count] = code_start;
        dict_count++;
    }
}

// Execute primitive
void exec_prim(int idx) {
    char* name = get_dict_name(idx);
    
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
        if (b != 0) push(a / b);
    }
    else if (str_eq(name, "MOD")) {
        int b = pop();
        int a = pop();
        if (b != 0) push(a % b);
    }
    // Comparison ops
    else if (str_eq(name, "=")) {
        int b = pop();
        int a = pop();
        push(a == b ? -1 : 0);  // Forth uses -1 for true
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
    // Stack ops
    else if (str_eq(name, "DUP")) {
        if (sp > 0) push(stack[sp-1]);
    }
    else if (str_eq(name, "DROP")) {
        pop();
    }
    else if (str_eq(name, "SWAP")) {
        if (sp >= 2) {
            int temp = stack[sp-1];
            stack[sp-1] = stack[sp-2];
            stack[sp-2] = temp;
        }
    }
    else if (str_eq(name, "OVER")) {
        if (sp >= 2) {
            push(stack[sp-2]);
        }
    }
    else if (str_eq(name, "ROT")) {
        if (sp >= 3) {
            int temp = stack[sp-3];
            stack[sp-3] = stack[sp-2];
            stack[sp-2] = stack[sp-1];
            stack[sp-1] = temp;
        }
    }
    else if (str_eq(name, "2DUP")) {
        if (sp >= 2) {
            push(stack[sp-2]);
            push(stack[sp-2]);
        }
    }
    // I/O
    else if (str_eq(name, ".")) {
        print_num(pop());
        putchar(' ');
    }
    else if (str_eq(name, "CR")) {
        putchar('\n');
    }
    else if (str_eq(name, ".S")) {
        puts("Stack:");
        for (int i = 0; i < sp; i++) {
            putchar(' ');
            print_num(stack[i]);
        }
        putchar('\n');
    }
    else if (str_eq(name, "WORDS")) {
        puts("Words:");
        for (int i = 0; i < dict_count; i++) {
            putchar(' ');
            puts(get_dict_name(i));
        }
        putchar('\n');
    }
    else if (str_eq(name, "BYE")) {
        running = 0;
    }
}

// Execute word
void execute(int idx) {
    if (idx < 0 || idx >= dict_count) return;
    
    if (dict_is_prim[idx]) {
        exec_prim(idx);
    } else {
        // Execute user-defined word
        int ip = dict_code_start[idx];
        while (code[ip] != -1) {
            if (code[ip] >= 0 && code[ip] < dict_count) {
                execute(code[ip]);
            } else if (code[ip] >= 10000) {
                push(code[ip] - 10000);
            }
            ip++;
        }
    }
}

// Initialize dictionary manually
void init_dict() {
    // Directly set names in the dictionary array to avoid issues with local buffers
    char* ptr;
    
    // +
    ptr = get_dict_name(dict_count);
    ptr[0] = '+'; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // -
    ptr = get_dict_name(dict_count);
    ptr[0] = '-'; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // *
    ptr = get_dict_name(dict_count);
    ptr[0] = '*'; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // /
    ptr = get_dict_name(dict_count);
    ptr[0] = '/'; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // MOD
    ptr = get_dict_name(dict_count);
    ptr[0] = 'M'; ptr[1] = 'O'; ptr[2] = 'D'; ptr[3] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;

    // =
    ptr = get_dict_name(dict_count);
    ptr[0] = '='; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;

    // <
    ptr = get_dict_name(dict_count);
    ptr[0] = '<'; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;

    // >
    ptr = get_dict_name(dict_count);
    ptr[0] = '>'; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;

    // DUP
    ptr = get_dict_name(dict_count);
    ptr[0] = 'D'; ptr[1] = 'U'; ptr[2] = 'P'; ptr[3] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // DROP
    ptr = get_dict_name(dict_count);
    ptr[0] = 'D'; ptr[1] = 'R'; ptr[2] = 'O'; ptr[3] = 'P'; ptr[4] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // SWAP
    ptr = get_dict_name(dict_count);
    ptr[0] = 'S'; ptr[1] = 'W'; ptr[2] = 'A'; ptr[3] = 'P'; ptr[4] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // OVER
    ptr = get_dict_name(dict_count);
    ptr[0] = 'O'; ptr[1] = 'V'; ptr[2] = 'E'; ptr[3] = 'R'; ptr[4] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;

    // ROT
    ptr = get_dict_name(dict_count);
    ptr[0] = 'R'; ptr[1] = 'O'; ptr[2] = 'T'; ptr[3] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;

    // 2DUP
    ptr = get_dict_name(dict_count);
    ptr[0] = '2'; ptr[1] = 'D'; ptr[2] = 'U'; ptr[3] = 'P'; ptr[4] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;

    // .
    ptr = get_dict_name(dict_count);
    ptr[0] = '.'; ptr[1] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // CR
    ptr = get_dict_name(dict_count);
    ptr[0] = 'C'; ptr[1] = 'R'; ptr[2] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // .S
    ptr = get_dict_name(dict_count);
    ptr[0] = '.'; ptr[1] = 'S'; ptr[2] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // WORDS
    ptr = get_dict_name(dict_count);
    ptr[0] = 'W'; ptr[1] = 'O'; ptr[2] = 'R'; ptr[3] = 'D'; ptr[4] = 'S'; ptr[5] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
    
    // BYE
    ptr = get_dict_name(dict_count);
    ptr[0] = 'B'; ptr[1] = 'Y'; ptr[2] = 'E'; ptr[3] = 0;
    dict_is_prim[dict_count] = 1;
    dict_code_start[dict_count] = 0;
    dict_count++;
}

// Process one word
void process_word(char* word) {
    // Check for colon definition
    if (str_eq(word, ":")) {
        compile_mode = 1;
        return;
    }
    
    if (str_eq(word, ";")) {
        if (compile_mode) {
            code[here++] = -1;
            compile_mode = 0;
        }
        return;
    }
    
    // Look up word
    int idx = find_word(word);
    if (idx >= 0) {
        if (compile_mode) {
            code[here++] = idx;
        } else {
            execute(idx);
        }
    } else {
        // Try as number
        int num;
        if (parse_num(word, &num)) {
            if (compile_mode) {
                code[here++] = num + 10000;
            } else {
                push(num);
            }
        } else if (compile_mode == 1) {
            // New word definition - copy directly to dictionary
            if (dict_count < MAX_WORDS) {
                char* dst = get_dict_name(dict_count);
                str_copy(dst, word);
                
                dict_is_prim[dict_count] = 0;
                dict_code_start[dict_count] = here;
                dict_count++;
                compile_mode = 2;
            }
        } else {
            puts("Unknown: ");
            puts(word);
        }
    }
}

// Get next word from input
int get_word(char* input, int* pos, char* word) {
    int i = 0;
    
    // Skip whitespace
    while (input[*pos] == ' ' || input[*pos] == '\t') {
        (*pos)++;
    }
    
    // Check end
    if (input[*pos] == 0 || input[*pos] == '\n') {
        return 0;
    }
    
    // Copy word
    while (i < 31) {
        char ch = input[*pos];
        
        if (!ch || ch == ' ' || ch == '\t' || ch == '\n') {
            break;
        }
        
        word[i] = ch;
        i++;
        (*pos)++;
    }
    word[i] = 0;
    
    return i > 0;
}

int main() {
    puts("Minimal Forth");
    puts("Arithmetic: + - * / MOD");
    puts("Comparison: = < >");
    puts("Stack: DUP DROP SWAP OVER ROT 2DUP");
    puts("I/O: . CR .S WORDS BYE");
    puts("Definition: : name ... ;");
    puts("");
    
    init_dict();
    
    char input[256];
    char word[32];
    
    while (running) {
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
                input[i] = 0;
                break;
            }
            putchar(ch);  // Echo
            input[i++] = ch;
        }
        input[i] = 0;
        
        // Process words
        int pos = 0;
        while (get_word(input, &pos, word)) {
            process_word(word);
        }
        
        if (!compile_mode) {
            puts(" ok");
        }
    }
    
    puts("Goodbye!");
    return 0;
}