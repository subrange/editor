// Tetris game for TEXT40 display
// Uses 1D arrays since 2D indexing doesn't work yet

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Game dimensions (leaving room for UI)
#define BOARD_WIDTH 10
#define BOARD_HEIGHT 20
#define BOARD_SIZE 200  // 10 * 20

// Board position on screen
#define BOARD_X 15
#define BOARD_Y 2

// Game state
unsigned char board[200];  // Fixed size: BOARD_WIDTH * BOARD_HEIGHT
int current_piece;
int current_x;
int current_y;
int current_rotation;
int score;
int lines_cleared;
int level;
int game_over;
int drop_timer;
int input_timer;

// Tetromino definitions (I, O, T, S, Z, J, L)
// Each piece has 4 rotations, each rotation is 4x4 = 16 cells
// Stored as flattened arrays (7 pieces * 4 rotations * 16 cells = 448 total)
unsigned char piece_data[448] = {
    // I piece - rotation 0
    0,0,0,0, 1,1,1,1, 0,0,0,0, 0,0,0,0,
    // I piece - rotation 1
    0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0,
    // I piece - rotation 2
    0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0,
    // I piece - rotation 3
    0,1,0,0, 0,1,0,0, 0,1,0,0, 0,1,0,0,
    
    // O piece - rotation 0
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    // O piece - rotation 1
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    // O piece - rotation 2
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    // O piece - rotation 3
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    
    // T piece - rotation 0
    0,1,0,0, 1,1,1,0, 0,0,0,0, 0,0,0,0,
    // T piece - rotation 1
    0,1,0,0, 0,1,1,0, 0,1,0,0, 0,0,0,0,
    // T piece - rotation 2
    0,0,0,0, 1,1,1,0, 0,1,0,0, 0,0,0,0,
    // T piece - rotation 3
    0,1,0,0, 1,1,0,0, 0,1,0,0, 0,0,0,0,
    
    // S piece - rotation 0
    0,1,1,0, 1,1,0,0, 0,0,0,0, 0,0,0,0,
    // S piece - rotation 1
    0,1,0,0, 0,1,1,0, 0,0,1,0, 0,0,0,0,
    // S piece - rotation 2
    0,0,0,0, 0,1,1,0, 1,1,0,0, 0,0,0,0,
    // S piece - rotation 3
    1,0,0,0, 1,1,0,0, 0,1,0,0, 0,0,0,0,
    
    // Z piece - rotation 0
    1,1,0,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    // Z piece - rotation 1
    0,0,1,0, 0,1,1,0, 0,1,0,0, 0,0,0,0,
    // Z piece - rotation 2
    0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0,
    // Z piece - rotation 3
    0,1,0,0, 1,1,0,0, 1,0,0,0, 0,0,0,0,
    
    // J piece - rotation 0
    1,0,0,0, 1,1,1,0, 0,0,0,0, 0,0,0,0,
    // J piece - rotation 1
    0,1,1,0, 0,1,0,0, 0,1,0,0, 0,0,0,0,
    // J piece - rotation 2
    0,0,0,0, 1,1,1,0, 0,0,1,0, 0,0,0,0,
    // J piece - rotation 3
    0,1,0,0, 0,1,0,0, 1,1,0,0, 0,0,0,0,
    
    // L piece - rotation 0
    0,0,1,0, 1,1,1,0, 0,0,0,0, 0,0,0,0,
    // L piece - rotation 1
    0,1,0,0, 0,1,0,0, 0,1,1,0, 0,0,0,0,
    // L piece - rotation 2
    0,0,0,0, 1,1,1,0, 1,0,0,0, 0,0,0,0,
    // L piece - rotation 3
    1,1,0,0, 0,1,0,0, 0,1,0,0, 0,0,0,0
};

// Piece colors
unsigned char piece_colors[7] = {
    COLOR_BLUE,      // I (using blue instead of cyan)
    COLOR_YELLOW,    // O
    COLOR_DARK_PURPLE, // T
    COLOR_GREEN,     // S
    COLOR_RED,       // Z
    COLOR_DARK_BLUE, // J (using dark blue)
    COLOR_ORANGE     // L
};

// Get random number in range
int rand_range(int max) {
    return (rng_get() % max);
}

// Convert 2D board position to 1D index
int board_index(int x, int y) {
    return y * BOARD_WIDTH + x;
}

// Get piece cell from 4x4 grid
int get_piece_cell(int piece, int rotation, int x, int y) {
    // Calculate index: piece * 64 + rotation * 16 + y * 4 + x
    int index = piece * 64 + rotation * 16 + y * 4 + x;
    return piece_data[index];
}

// Check if position is valid for current piece
int is_valid_position(int piece, int x, int y, int rotation) {
    for (int py = 0; py < 4; py++) {
        for (int px = 0; px < 4; px++) {
            if (get_piece_cell(piece, rotation, px, py)) {
                int board_x = x + px;
                int board_y = y + py;
                
                // Check boundaries
                if (board_x < 0 || board_x >= BOARD_WIDTH || 
                    board_y < 0 || board_y >= BOARD_HEIGHT) {
                    return 0;
                }
                
                // Check collision with placed pieces
                if (board[board_index(board_x, board_y)]) {
                    return 0;
                }
            }
        }
    }
    return 1;
}

// Place piece on board
void place_piece() {
    for (int py = 0; py < 4; py++) {
        for (int px = 0; px < 4; px++) {
            if (get_piece_cell(current_piece, current_rotation, px, py)) {
                int board_x = current_x + px;
                int board_y = current_y + py;
                if (board_x >= 0 && board_x < BOARD_WIDTH && 
                    board_y >= 0 && board_y < BOARD_HEIGHT) {
                    board[board_index(board_x, board_y)] = current_piece + 1;
                }
            }
        }
    }
}

// Check and clear completed lines
int clear_lines() {
    int lines = 0;
    
    for (int y = BOARD_HEIGHT - 1; y >= 0; y--) {
        int complete = 1;
        for (int x = 0; x < BOARD_WIDTH; x++) {
            if (!board[board_index(x, y)]) {
                complete = 0;
                break;
            }
        }
        
        if (complete) {
            // Move everything down
            for (int move_y = y; move_y > 0; move_y--) {
                for (int x = 0; x < BOARD_WIDTH; x++) {
                    board[board_index(x, move_y)] = board[board_index(x, move_y - 1)];
                }
            }
            // Clear top line
            for (int x = 0; x < BOARD_WIDTH; x++) {
                board[board_index(x, 0)] = 0;
            }
            lines++;
            y++; // Check this line again
        }
    }
    
    return lines;
}

// Spawn new piece
void spawn_piece() {
    current_piece = rand_range(7);
    current_x = 3;
    current_y = 0;
    current_rotation = 0;
    
    // Check game over
    if (!is_valid_position(current_piece, current_x, current_y, current_rotation)) {
        game_over = 1;
    }
}

// Initialize game
void init_game() {
    // Clear board
    for (int i = 0; i < BOARD_SIZE; i++) {
        board[i] = 0;
    }
    
    score = 0;
    lines_cleared = 0;
    level = 1;
    game_over = 0;
    drop_timer = 0;
    input_timer = 0;
    
    spawn_piece();
}

// Move piece
int move_piece(int dx, int dy) {
    if (is_valid_position(current_piece, current_x + dx, current_y + dy, current_rotation)) {
        current_x += dx;
        current_y += dy;
        return 1;
    }
    return 0;
}

// Rotate piece
void rotate_piece() {
    int new_rotation = (current_rotation + 1) % 4;
    if (is_valid_position(current_piece, current_x, current_y, new_rotation)) {
        current_rotation = new_rotation;
    }
}

// Drop piece one line
int drop_piece() {
    if (move_piece(0, 1)) {
        return 1;
    } else {
        // Piece landed
        place_piece();
        int lines = clear_lines();
        if (lines > 0) {
            lines_cleared += lines;
            // Score: 100 for 1 line, 300 for 2, 500 for 3, 800 for 4
            if (lines == 1) score += 100 * level;
            else if (lines == 2) score += 300 * level;
            else if (lines == 3) score += 500 * level;
            else if (lines == 4) score += 800 * level;
            
            // Level up every 10 lines
            level = 1 + (lines_cleared / 10);
        }
        spawn_piece();
        return 0;
    }
}

// Draw the game board
void draw_board() {
    // Draw board background and pieces
    for (int y = 0; y < BOARD_HEIGHT; y++) {
        for (int x = 0; x < BOARD_WIDTH; x++) {
            int piece_type = board[board_index(x, y)];
            
            if (piece_type > 0) {
                // Placed piece
                unsigned char color = piece_colors[piece_type - 1];
                text40_putchar_color(BOARD_X + x, BOARD_Y + y, ' ', COLOR_BLACK, color);
            } else {
                // Empty cell
                text40_putchar_color(BOARD_X + x, BOARD_Y + y, '.', COLOR_DARK_GRAY, COLOR_BLACK);
            }
        }
    }
    
    // Draw current piece
    for (int py = 0; py < 4; py++) {
        for (int px = 0; px < 4; px++) {
            if (get_piece_cell(current_piece, current_rotation, px, py)) {
                int screen_x = BOARD_X + current_x + px;
                int screen_y = BOARD_Y + current_y + py;
                if (screen_x >= BOARD_X && screen_x < BOARD_X + BOARD_WIDTH &&
                    screen_y >= BOARD_Y && screen_y < BOARD_Y + BOARD_HEIGHT) {
                    unsigned char color = piece_colors[current_piece];
                    text40_putchar_color(screen_x, screen_y, ' ', COLOR_BLACK, color);
                }
            }
        }
    }
    
    // Draw ghost piece (preview where piece will land)
    int ghost_y = current_y;
    while (is_valid_position(current_piece, current_x, ghost_y + 1, current_rotation)) {
        ghost_y++;
    }
    if (ghost_y != current_y) {
        for (int py = 0; py < 4; py++) {
            for (int px = 0; px < 4; px++) {
                if (get_piece_cell(current_piece, current_rotation, px, py)) {
                    int screen_x = BOARD_X + current_x + px;
                    int screen_y = BOARD_Y + ghost_y + py;
                    if (screen_x >= BOARD_X && screen_x < BOARD_X + BOARD_WIDTH &&
                        screen_y >= BOARD_Y && screen_y < BOARD_Y + BOARD_HEIGHT &&
                        !board[board_index(current_x + px, ghost_y + py)]) {
                        text40_putchar_color(screen_x, screen_y, '+', COLOR_DARK_GRAY, COLOR_BLACK);
                    }
                }
            }
        }
    }
}

// Draw UI elements
void draw_ui() {
    // Title
    text40_puts_color(15, 0, " T E T R I S ", COLOR_WHITE, COLOR_DARK_BLUE);
    
    // Board border
    for (int y = BOARD_Y - 1; y <= BOARD_Y + BOARD_HEIGHT; y++) {
        text40_putchar_color(BOARD_X - 1, y, '|', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(BOARD_X + BOARD_WIDTH, y, '|', COLOR_BLUE, COLOR_BLACK);
    }
    for (int x = BOARD_X - 1; x <= BOARD_X + BOARD_WIDTH; x++) {
        text40_putchar_color(x, BOARD_Y - 1, '=', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(x, BOARD_Y + BOARD_HEIGHT, '=', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Score panel
    text40_puts_color(2, 3, "SCORE:", COLOR_WHITE, COLOR_BLACK);
    char score_str[10];
    int temp = score;
    for (int i = 9; i >= 0; i--) {
        score_str[i] = '0' + (temp % 10);
        temp /= 10;
    }
    for (int i = 4; i < 10; i++) {
        if (score_str[i] != '0' || i == 9) {
            for (int j = i; j < 10; j++) {
                text40_putchar_color(2 + j - i, 4, score_str[j], COLOR_YELLOW, COLOR_BLACK);
            }
            break;
        }
    }
    
    // Lines
    text40_puts_color(2, 6, "LINES:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(2, 7, '0' + (lines_cleared / 100) % 10, COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(3, 7, '0' + (lines_cleared / 10) % 10, COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(4, 7, '0' + lines_cleared % 10, COLOR_GREEN, COLOR_BLACK);
    
    // Level
    text40_puts_color(2, 9, "LEVEL:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(2, 10, '0' + (level / 10) % 10, COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(3, 10, '0' + level % 10, COLOR_BLUE, COLOR_BLACK);
    
    // Controls
    text40_puts_color(2, 13, "CONTROLS:", COLOR_LIGHT_GRAY, COLOR_BLACK);
    text40_puts_color(2, 14, "A/D: Move", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(2, 15, "W: Rotate", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(2, 16, "S: Drop", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(2, 17, "Space: Fall", COLOR_DARK_GRAY, COLOR_BLACK);
    
    // Next piece preview
    text40_puts_color(28, 3, "NEXT:", COLOR_WHITE, COLOR_BLACK);
    // Would show next piece here, but keeping it simple
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Get input (simplified - would need proper input handling)
char get_input() {
    // In a real implementation, this would check for keyboard input
    // For demo, return random moves occasionally
    if (rand_range(100) < 5) {
        int choice = rand_range(5);
        if (choice == 0) return 'a';
        if (choice == 1) return 'd';
        if (choice == 2) return 'w';
        if (choice == 3) return 's';
        return ' ';
    }
    return 0;
}

int main() {
    // Enable TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Initialize game
    init_game();
    
    // Game loop
    int frame = 0;
    int drop_speed = 30; // Frames between automatic drops
    
    while (!game_over && frame < 1000) { // Limited frames for demo
        // Clear display
        display_clear();
        
        // Handle input (simplified for demo)
        if (input_timer == 0) {
            char input = get_input();
            if (input == 'a') {
                move_piece(-1, 0);
                input_timer = 3;
            } else if (input == 'd') {
                move_piece(1, 0);
                input_timer = 3;
            } else if (input == 'w') {
                rotate_piece();
                input_timer = 5;
            } else if (input == 's') {
                drop_piece();
                drop_timer = 0;
                input_timer = 2;
            } else if (input == ' ') {
                // Hard drop
                while (drop_piece()) {
                    score += 2;
                }
                input_timer = 10;
            }
        }
        if (input_timer > 0) input_timer--;
        
        // Automatic drop
        drop_timer++;
        if (drop_timer >= drop_speed) {
            drop_piece();
            drop_timer = 0;
        }
        
        // Increase speed with level
        drop_speed = 30 - (level * 2);
        if (drop_speed < 5) drop_speed = 5;
        
        // Draw everything
        draw_ui();
        draw_board();
        
        // Update display
        display_flush();
        
        // Frame delay
        delay(5000);
        frame++;
    }
    
    // Game over screen
    if (game_over) {
        text40_puts_color(12, 11, " GAME OVER! ", COLOR_WHITE, COLOR_RED);
        text40_puts_color(11, 13, " FINAL SCORE ", COLOR_BLACK, COLOR_YELLOW);
        display_flush();
        delay(50000);
    } else {
        text40_puts_color(12, 11, " DEMO COMPLETE ", COLOR_WHITE, COLOR_GREEN);
        display_flush();
        delay(30000);
    }
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Tetris demo completed!");
    
    return 0;
}