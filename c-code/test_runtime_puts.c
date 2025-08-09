// Test runtime puts function
extern int puts(char *s);

int main() {
    char msg[6];
    msg[0] = 'T';
    msg[1] = 'e';
    msg[2] = 's';
    msg[3] = 't';
    msg[4] = '!';
    msg[5] = 0;
    
    puts(msg);
    return 0;
}