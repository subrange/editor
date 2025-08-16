#include <stdio.h>
// --- minimal decimal printer (works for any non-negative int) ---
void put_dec(int n) {
    if (n >= 10) put_dec(n / 10);
    putchar('0' + (n % 10));
}

// --- ANSI helpers built from raw bytes ---
void esc() { putchar(27); }                 // ESC
void csi() { esc(); putchar('['); }         // ESC[

void enter_alt()  { csi(); putchar('?'); put_dec(1049); putchar('h'); } // alt screen on
void leave_alt()  { csi(); putchar('?'); put_dec(1049); putchar('l'); } // alt screen off

void hide_cursor(){ csi(); putchar('?'); put_dec(25);   putchar('l'); }
void show_cursor(){ csi(); putchar('?'); put_dec(25);   putchar('h'); }

void clear_screen(){ csi(); put_dec(2); putchar('J'); } // clear
void home(){ csi(); putchar('H'); }                     // row=1 col=1

void move_to(int row, int col) {
    csi(); put_dec(row); putchar(';'); put_dec(col); putchar('H');
}

void clear_line() { csi(); put_dec(2); putchar('K'); }  // erase whole line

// --- tiny stdout-only demo: bouncing '@' ---
void tiny_demo() {
    int row = 1;
    int col = 1;
    int dr = 1;
    int dc = 1;
    int maxr = 20;
    int maxc = 40;
    int t = 0;

    enter_alt();
    hide_cursor();
    clear_screen();
    home();

    while (t < 800) {
        move_to(row, col); putchar(' ');     // erase previous
        row += dr; col += dc;
        if (row <= 1 || row >= maxr) dr = -dr;
        if (col <= 1 || col >= maxc) dc = -dc;
        move_to(row, col); putchar('@');     // draw
        move_to(maxr + 1, 1);               // park cursor

        // crude delay
        int k = 0; while (k < 30000) { k = k + 1; }

        t = t + 1;
    }

    show_cursor();
    leave_alt();
}

int main() {
    tiny_demo();
    return 0;
}