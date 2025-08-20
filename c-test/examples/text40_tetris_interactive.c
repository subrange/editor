// Interactive Tetris game for TEXT40 display with keyboard controls
// Arrow keys: move left/right, rotate up, drop down
// Z: soft drop, X: hard drop

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Game dimensions
#define BOARD_WIDTH 10
#define BOARD_HEIGHT 20
#define BOARD_SIZE 200

// Board position on screen
#define BOARD_X 15
#define BOARD_Y 2

// Game state
unsigned char board[200];
int current_piece;
int current_x;
int current_y;
int current_rotation;
int score;
int lines_cleared;
int level;
int game_over;
int drop_timer;
int frames_alive;

// Tetromino definitions (I, O, T, S, Z, J, L)
unsigned char piece_data[448] = {
    // I piece - rotation 0
    0,0,0,0, 1,1,1,1, 0,0,0,0, 0,0,0,0,
    // I piece - rotation 1
    0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0,
    // I piece - rotation 2
    0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0,
    // I piece - rotation 3
    0,1,0,0, 0,1,0,0, 0,1,0,0, 0,1,0,0,
    
    // O piece - all rotations same
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    0,1,1,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    
    // T piece
    0,1,0,0, 1,1,1,0, 0,0,0,0, 0,0,0,0,
    0,1,0,0, 0,1,1,0, 0,1,0,0, 0,0,0,0,
    0,0,0,0, 1,1,1,0, 0,1,0,0, 0,0,0,0,
    0,1,0,0, 1,1,0,0, 0,1,0,0, 0,0,0,0,
    
    // S piece
    0,1,1,0, 1,1,0,0, 0,0,0,0, 0,0,0,0,
    0,1,0,0, 0,1,1,0, 0,0,1,0, 0,0,0,0,
    0,0,0,0, 0,1,1,0, 1,1,0,0, 0,0,0,0,
    1,0,0,0, 1,1,0,0, 0,1,0,0, 0,0,0,0,
    
    // Z piece
    1,1,0,0, 0,1,1,0, 0,0,0,0, 0,0,0,0,
    0,0,1,0, 0,1,1,0, 0,1,0,0, 0,0,0,0,
    0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0,
    0,1,0,0, 1,1,0,0, 1,0,0,0, 0,0,0,0,
    
    // J piece
    1,0,0,0, 1,1,1,0, 0,0,0,0, 0,0,0,0,
    0,1,1,0, 0,1,0,0, 0,1,0,0, 0,0,0,0,
    0,0,0,0, 1,1,1,0, 0,0,1,0, 0,0,0,0,
    0,1,0,0, 0,1,0,0, 1,1,0,0, 0,0,0,0,
    
    // L piece
    0,0,1,0, 1,1,1,0, 0,0,0,0, 0,0,0,0,
    0,1,0,0, 0,1,0,0, 0,1,1,0, 0,0,0,0,
    0,0,0,0, 1,1,1,0, 1,0,0,0, 0,0,0,0,
    1,1,0,0, 0,1,0,0, 0,1,0,0, 0,0,0,0
};

// Piece colors
unsigned char piece_colors[7] = {
    COLOR_BLUE,      // I
    COLOR_YELLOW,    // O
    COLOR_DARK_PURPLE, // T
    COLOR_GREEN,     // S
    COLOR_RED,       // Z
    COLOR_DARK_BLUE, // J
    COLOR_ORANGE     // L
};

// Get random number in range
int rand_range(int max) {
    return (rng_get() % max);
}

// Convert 2D to 1D index
int board_index(int x, int y) {
    return y * BOARD_WIDTH + x;
}

// Get piece cell
int get_piece_cell(int piece, int rotation, int x, int y) {
    int index = piece * 64 + rotation * 16 + y * 4 + x;
    return piece_data[index];
}

// Check if position is valid
int is_valid_position(int piece, int x, int y, int rotation) {
    for (int py = 0; py < 4; py++) {
        for (int px = 0; px < 4; px++) {
            if (get_piece_cell(piece, rotation, px, py)) {
                int board_x = x + px;
                int board_y = y + py;
                
                if (board_x < 0 || board_x >= BOARD_WIDTH || 
                    board_y < 0 || board_y >= BOARD_HEIGHT) {
                    return 0;
                }
                
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

// Clear completed lines
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
    
    if (!is_valid_position(current_piece, current_x, current_y, current_rotation)) {
        game_over = 1;
    }
}

// Initialize game
void init_game() {
    for (int i = 0; i < BOARD_SIZE; i++) {
        board[i] = 0;
    }
    
    score = 0;
    lines_cleared = 0;
    level = 1;
    game_over = 0;
    drop_timer = 0;
    frames_alive = 0;
    
    // Seed RNG with something dynamic
    rng_set_seed(12345);
    
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
            // Scoring
            if (lines == 1) score += 100 * level;
            else if (lines == 2) score += 300 * level;
            else if (lines == 3) score += 500 * level;
            else if (lines == 4) score += 800 * level;
            
            // Level up every 10 lines
            level = 1 + (lines_cleared / 10);
            if (level > 15) level = 15;
        }
        spawn_piece();
        return 0;
    }
}

// Draw the game board
void draw_board() {
    // Draw placed pieces
    for (int y = 0; y < BOARD_HEIGHT; y++) {
        for (int x = 0; x < BOARD_WIDTH; x++) {
            int piece_type = board[board_index(x, y)];
            
            if (piece_type > 0) {
                unsigned char color = piece_colors[piece_type - 1];
                text40_putchar_color(BOARD_X + x, BOARD_Y + y, ' ', COLOR_BLACK, color);
            } else {
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
    
    // Draw ghost piece
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

// Draw UI
void draw_ui() {
    // Title
    text40_puts_color(13, 0, "INTERACTIVE TETRIS", COLOR_WHITE, COLOR_DARK_BLUE);
    
    // Board border
    for (int y = BOARD_Y - 1; y <= BOARD_Y + BOARD_HEIGHT; y++) {
        text40_putchar_color(BOARD_X - 1, y, '|', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(BOARD_X + BOARD_WIDTH, y, '|', COLOR_BLUE, COLOR_BLACK);
    }
    for (int x = BOARD_X - 1; x <= BOARD_X + BOARD_WIDTH; x++) {
        text40_putchar_color(x, BOARD_Y - 1, '=', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(x, BOARD_Y + BOARD_HEIGHT, '=', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Score
    text40_puts_color(2, 3, "SCORE:", COLOR_WHITE, COLOR_BLACK);
    // Display score digits
    int temp_score = score;
    for (int i = 0; i < 7; i++) {
        text40_putchar_color(8 - i, 4, '0' + (temp_score % 10), COLOR_YELLOW, COLOR_BLACK);
        temp_score /= 10;
        if (temp_score == 0) break;
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
    text40_puts_color(2, 14, "← → Move", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(2, 15, "↑ Rotate", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(2, 16, "↓ Soft Drop", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(2, 17, "Z: Fast Drop", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(2, 18, "X: Hard Drop", COLOR_DARK_GRAY, COLOR_BLACK);
    
    // Show key states for debugging
    text40_puts_color(28, 3, "KEYS:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(28, 4, key_up_pressed() ? '^' : '.', 
                        key_up_pressed() ? COLOR_GREEN : COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(27, 5, key_left_pressed() ? '<' : '.', 
                        key_left_pressed() ? COLOR_GREEN : COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(28, 5, key_down_pressed() ? 'v' : '.', 
                        key_down_pressed() ? COLOR_GREEN : COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(29, 5, key_right_pressed() ? '>' : '.', 
                        key_right_pressed() ? COLOR_GREEN : COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(27, 6, key_z_pressed() ? 'Z' : '.', 
                        key_z_pressed() ? COLOR_GREEN : COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(29, 6, key_x_pressed() ? 'X' : '.', 
                        key_x_pressed() ? COLOR_GREEN : COLOR_DARK_GRAY, COLOR_BLACK);
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

int main() {
    // Enable TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Initialize game
    init_game();
    
    // Game state
    int drop_speed = 30;
    int input_delay = 0;
    int last_left = 0;
    int last_right = 0;
    int last_up = 0;
    int last_down = 0;
    int last_z = 0;
    int last_x = 0;
    
    // Main game loop
    while (!game_over && frames_alive < 5000) {
        // Clear display
        display_clear();
        
        // Read keyboard input
        int left = key_left_pressed();
        int right = key_right_pressed();
        int up = key_up_pressed();
        int down = key_down_pressed();
        int z = key_z_pressed();
        int x = key_x_pressed();
        
        // Handle input with delay to prevent too rapid movement
        if (input_delay == 0) {
            // Move left
            if (left && !last_left) {
                move_piece(-1, 0);
                input_delay = 3;
            }
            // Move right
            else if (right && !last_right) {
                move_piece(1, 0);
                input_delay = 3;
            }
            // Rotate
            else if (up && !last_up) {
                rotate_piece();
                input_delay = 5;
            }
            // Soft drop
            else if (down) {
                if (drop_piece()) {
                    score += 1;
                }
                drop_timer = 0;
                input_delay = 2;
            }
            // Fast drop (Z)
            else if (z && !last_z) {
                for (int i = 0; i < 3; i++) {
                    if (drop_piece()) {
                        score += 1;
                    } else {
                        break;
                    }
                }
                input_delay = 5;
            }
            // Hard drop (X)
            else if (x && !last_x) {
                while (drop_piece()) {
                    score += 2;
                }
                input_delay = 10;
            }
        }
        
        // Store last key states
        last_left = left;
        last_right = right;
        last_up = up;
        last_down = down;
        last_z = z;
        last_x = x;
        
        // Decrease input delay
        if (input_delay > 0) {
            input_delay--;
        }
        
        // Automatic drop
        drop_timer++;
        if (drop_timer >= drop_speed) {
            drop_piece();
            drop_timer = 0;
        }
        
        // Calculate drop speed based on level
        drop_speed = 30 - (level * 2);
        if (drop_speed < 3) drop_speed = 3;
        
        // Draw everything
        draw_ui();
        draw_board();
        
        // Update display
        display_flush();
        
        // Frame delay
        delay(3000);
        frames_alive++;
    }
    
    // Game over screen
    if (game_over) {
        text40_puts_color(12, 11, " GAME OVER! ", COLOR_WHITE, COLOR_RED);
        text40_puts_color(10, 13, " FINAL SCORE: ", COLOR_BLACK, COLOR_YELLOW);
        // Show final score
        int temp = score;
        for (int i = 0; i < 7; i++) {
            text40_putchar_color(24 - i, 13, '0' + (temp % 10), COLOR_BLACK, COLOR_YELLOW);
            temp /= 10;
            if (temp == 0) break;
        }
    } else {
        text40_puts_color(10, 11, " TIME'S UP! ", COLOR_WHITE, COLOR_ORANGE);
        text40_puts_color(8, 13, " THANKS FOR PLAYING! ", COLOR_BLACK, COLOR_GREEN);
    }
    
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Interactive Tetris completed!");
    
    return 0;
}