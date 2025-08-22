// Fully Interactive Tetris for RGB565 64x64 display
// This version has actual piece movement and rotation

#include <graphics.h>
#include <mmio.h>

// Game dimensions
#define BOARD_WIDTH 10
#define BOARD_HEIGHT 20
#define BLOCK_SIZE 3

// Board position on screen
#define BOARD_X 17
#define BOARD_Y 2

// Get piece shape data with rotation
int get_piece_data(int piece, int rotation, int idx) {
    int x = idx % 4;
    int y = idx / 4;
    
    // Apply rotation transformation (CLOCKWISE)
    if (rotation == 1) {
        // 90 degrees clockwise: (x,y) -> (y, 3-x)
        int nx = y;
        int ny = 3 - x;
        x = nx;
        y = ny;
        idx = y * 4 + x;
    } else if (rotation == 2) {
        // 180 degrees: (x,y) -> (3-x, 3-y)
        x = 3 - x;
        y = 3 - y;
        idx = y * 4 + x;
    } else if (rotation == 3) {
        // 270 degrees clockwise: (x,y) -> (3-y, x)
        int nx = 3 - y;
        int ny = x;
        x = nx;
        y = ny;
        idx = y * 4 + x;
    }
    
    // I piece: horizontal line
    if (piece == 0) {
        if (idx == 4 || idx == 5 || idx == 6 || idx == 7) return 1;
        return 0;
    }
    // O piece: square (no rotation needed)
    if (piece == 1) {
        if (idx == 5 || idx == 6 || idx == 9 || idx == 10) return 1;
        return 0;
    }
    // T piece
    if (piece == 2) {
        if (idx == 5 || idx == 8 || idx == 9 || idx == 10) return 1;
        return 0;
    }
    // S piece
    if (piece == 3) {
        if (idx == 5 || idx == 6 || idx == 8 || idx == 9) return 1;
        return 0;
    }
    // Z piece
    if (piece == 4) {
        if (idx == 4 || idx == 5 || idx == 9 || idx == 10) return 1;
        return 0;
    }
    // J piece
    if (piece == 5) {
        if (idx == 4 || idx == 8 || idx == 9 || idx == 10) return 1;
        return 0;
    }
    // L piece
    if (piece == 6) {
        if (idx == 6 || idx == 8 || idx == 9 || idx == 10) return 1;
        return 0;
    }
    return 0;
}

// Get piece color by type (0-6)
unsigned short get_piece_color(int piece) {
    if (piece == 0) return RGB_CYAN;     // I
    if (piece == 1) return RGB_YELLOW;   // O  
    if (piece == 2) return RGB_MAGENTA;  // T
    if (piece == 3) return RGB_GREEN;    // S
    if (piece == 4) return RGB_RED;      // Z
    if (piece == 5) return RGB_BLUE;     // J
    if (piece == 6) return RGB_ORANGE;   // L
    // Default case - should not happen
    return RGB_WHITE;
}

// Get color for board value (1-7 maps to piece types 0-6)
unsigned short get_board_color(int board_value) {
    if (board_value <= 0) return RGB_BLACK;  // Empty
    if (board_value > 7) return RGB_WHITE;   // Error case
    return get_piece_color(board_value - 1);
}

// Get piece cell with rotation
int get_piece_cell(int piece, int rotation, int x, int y) {
    if (x < 0 || x >= 4 || y < 0 || y >= 4) return 0;
    return get_piece_data(piece, rotation, y * 4 + x);
}

// Check if position is valid with rotation
int is_valid_position(unsigned char *board, int piece, int rotation, int px, int py) {
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            if (get_piece_cell(piece, rotation, x, y)) {
                int board_x = px + x;
                int board_y = py + y;
                
                // Check boundaries
                if (board_x < 0 || board_x >= BOARD_WIDTH || 
                    board_y < 0 || board_y >= BOARD_HEIGHT) {
                    return 0;
                }
                
                // Check collision with placed pieces
                if (board[board_y * BOARD_WIDTH + board_x]) {
                    return 0;
                }
            }
        }
    }
    return 1;
}

// Place piece on board with rotation
void place_piece(unsigned char *board, int piece, int rotation, int px, int py) {
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            if (get_piece_cell(piece, rotation, x, y)) {
                int board_x = px + x;
                int board_y = py + y;
                if (board_x >= 0 && board_x < BOARD_WIDTH && 
                    board_y >= 0 && board_y < BOARD_HEIGHT) {
                    // Store piece type + 1 to distinguish from empty (0)
                    // This means board contains values 1-7 for pieces 0-6
                    board[board_y * BOARD_WIDTH + board_x] = piece + 1;
                }
            }
        }
    }
}

// Clear completed lines
int clear_lines(unsigned char *board) {
    int lines_cleared = 0;
    
    for (int y = BOARD_HEIGHT - 1; y >= 0; y--) {
        int complete = 1;
        for (int x = 0; x < BOARD_WIDTH; x++) {
            if (!board[y * BOARD_WIDTH + x]) {
                complete = 0;
                break;
            }
        }
        
        if (complete) {
            // Move everything down
            for (int move_y = y; move_y > 0; move_y--) {
                for (int x = 0; x < BOARD_WIDTH; x++) {
                    board[move_y * BOARD_WIDTH + x] = 
                        board[(move_y - 1) * BOARD_WIDTH + x];
                }
            }
            // Clear top line
            for (int x = 0; x < BOARD_WIDTH; x++) {
                board[x] = 0;
            }
            lines_cleared++;
            y++; // Check this line again
        }
    }
    
    return lines_cleared;
}

// Draw a block
void draw_block(int x, int y, unsigned short color) {
    for (int dy = 0; dy < BLOCK_SIZE; dy++) {
        for (int dx = 0; dx < BLOCK_SIZE; dx++) {
            set_pixel(x + dx, y + dy, color);
        }
    }
}

// Draw the board
void draw_board(unsigned char *board) {
    // Draw board background
    fill_rect(BOARD_X - 1, BOARD_Y - 1,
              BOARD_WIDTH * BLOCK_SIZE + 2,
              BOARD_HEIGHT * BLOCK_SIZE + 2,
              rgb565(40, 40, 60));
    
    fill_rect(BOARD_X, BOARD_Y,
              BOARD_WIDTH * BLOCK_SIZE,
              BOARD_HEIGHT * BLOCK_SIZE,
              rgb565(10, 10, 20));
    
    // Draw placed pieces
    for (int y = 0; y < BOARD_HEIGHT; y++) {
        for (int x = 0; x < BOARD_WIDTH; x++) {
            int board_value = board[y * BOARD_WIDTH + x];
            if (board_value > 0) {
                unsigned short color = get_board_color(board_value);
                draw_block(BOARD_X + x * BLOCK_SIZE,
                          BOARD_Y + y * BLOCK_SIZE,
                          color);
            }
        }
    }
}

// Draw current piece with rotation
void draw_current_piece(int piece, int rotation, int px, int py, unsigned short color) {
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            if (get_piece_cell(piece, rotation, x, y)) {
                draw_block(BOARD_X + (px + x) * BLOCK_SIZE,
                          BOARD_Y + (py + y) * BLOCK_SIZE,
                          color);
            }
        }
    }
}

// Draw ghost piece (shows where piece will land) with rotation
void draw_ghost_piece(unsigned char *board, int piece, int rotation, int px, int py) {
    int ghost_y = py;
    while (is_valid_position(board, piece, rotation, px, ghost_y + 1)) {
        ghost_y++;
    }
    
    if (ghost_y != py) {
        for (int y = 0; y < 4; y++) {
            for (int x = 0; x < 4; x++) {
                if (get_piece_cell(piece, rotation, x, y)) {
                    int screen_x = BOARD_X + (px + x) * BLOCK_SIZE;
                    int screen_y = BOARD_Y + (ghost_y + y) * BLOCK_SIZE;
                    // Draw ghost piece as bright outline
                    draw_rect(screen_x, screen_y, BLOCK_SIZE, BLOCK_SIZE,
                             rgb565(150, 150, 150));  // Much brighter gray
                    // Add inner dark fill to make it look hollow
                    fill_rect(screen_x + 1, screen_y + 1, BLOCK_SIZE - 2, BLOCK_SIZE - 2,
                             rgb565(20, 20, 30));
                }
            }
        }
    }
}

// Draw next piece preview
void draw_next_piece(int piece) {
    // Next piece preview box
    fill_rect(50, 2, 12, 12, rgb565(30, 30, 50));
    fill_rect(51, 3, 10, 10, rgb565(10, 10, 20));
    
    // Draw the next piece in the preview box (smaller scale)
    unsigned short color = get_piece_color(piece);
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            if (get_piece_cell(piece, 0, x, y)) {
                // Draw smaller blocks (2x2 instead of 3x3)
                fill_rect(52 + x * 2, 4 + y * 2, 2, 2, color);
            }
        }
    }
    
    // "NEXT" label
    set_pixel(51, 1, RGB_WHITE);
    set_pixel(53, 1, RGB_WHITE);
    set_pixel(55, 1, RGB_WHITE);
    set_pixel(57, 1, RGB_WHITE);
}

// Draw UI
void draw_ui(int score, int lines, int level) {
    // Score area
    fill_rect(2, 10, 12, 30, rgb565(30, 30, 50));
    int score_bars = (score / 100) % 10;
    for (int i = 0; i < score_bars; i++) {
        fill_rect(3, 12 + i * 3, 10, 2, RGB_YELLOW);
    }
    
    // Lines cleared indicator
    fill_rect(2, 45, 12, 15, rgb565(30, 30, 50));
    int line_bars = lines % 5;
    for (int i = 0; i < line_bars; i++) {
        fill_rect(3, 47 + i * 3, 10, 2, RGB_CYAN);
    }
    
    // Level indicator
    fill_rect(50, 16, 12, 20, rgb565(30, 30, 50));
    int level_height = (level * 2);
    if (level_height > 18) level_height = 18;
    fill_rect(51, 34 - level_height, 10, level_height, RGB_GREEN);
    
    // Control hints
    fill_rect(50, 39, 12, 21, rgb565(30, 30, 50));
    // Arrow indicators
    set_pixel(55, 42, RGB_WHITE); // Up
    set_pixel(54, 43, RGB_WHITE);
    set_pixel(56, 43, RGB_WHITE);
    
    set_pixel(52, 47, RGB_WHITE); // Left
    set_pixel(53, 46, RGB_WHITE);
    set_pixel(53, 48, RGB_WHITE);
    
    set_pixel(58, 47, RGB_WHITE); // Right
    set_pixel(57, 46, RGB_WHITE);
    set_pixel(57, 48, RGB_WHITE);
    
    set_pixel(55, 52, RGB_WHITE); // Down
    set_pixel(54, 51, RGB_WHITE);
    set_pixel(56, 51, RGB_WHITE);
    
    // X indicator
    set_pixel(53, 56, RGB_WHITE);
    set_pixel(54, 57, RGB_WHITE);
    set_pixel(55, 58, RGB_WHITE);
    set_pixel(56, 57, RGB_WHITE);
    set_pixel(57, 56, RGB_WHITE);
}

int main() {
    // Initialize display
    graphics_init(64, 64);
    
    // Initialize RNG with a more random seed
    // Use initial keyboard state as entropy source
    unsigned short seed = 12345;
    seed = seed ^ (key_left_pressed() << 1);
    seed = seed ^ (key_right_pressed() << 2);
    seed = seed ^ (key_up_pressed() << 3);
    seed = seed ^ (key_down_pressed() << 4);
    seed = seed ^ (key_z_pressed() << 5);
    seed = seed ^ (key_x_pressed() << 6);
    rng_set_seed(seed);
    
    // Local game state (avoiding globals)
    unsigned char board[200];
    
    // Clear board
    for (int i = 0; i < 200; i++) {
        board[i] = 0;
    }
    
    // Game variables
    int current_piece = rng_get() % 7;
    int next_piece = rng_get() % 7;  // Next piece preview
    int current_x = 3;
    int current_y = 0;
    int current_rotation = 0;
    int score = 0;
    int lines = 0;
    int level = 1;
    int drop_timer = 0;
    int game_over = 0;
    
    // Input tracking
    int last_left = 0;
    int last_right = 0;
    int last_up = 0;
    int last_down = 0;
    int last_z = 0;
    int last_x = 0;
    int move_delay = 0;
    int repeat_delay = 0;  // For key repeat
    
    // Main game loop
    for (int frame = 0; frame < 5000 && !game_over; frame++) {
        // Clear screen
        clear_screen(rgb565(5, 5, 15));
        
        // Handle input
        int left = key_left_pressed();
        int right = key_right_pressed();
        int up = key_up_pressed();
        int down = key_down_pressed();
        int z = key_z_pressed();
        int x = key_x_pressed();
        
        if (move_delay == 0) {
            // Move left (initial press or repeat)
            if (left) {
                if (!last_left || repeat_delay == 0) {
                    if (is_valid_position(board, current_piece, current_rotation, current_x - 1, current_y)) {
                        current_x--;
                        move_delay = 3;
                        repeat_delay = last_left ? 2 : 8;  // Faster repeat after initial delay
                    }
                }
            }
            // Move right (initial press or repeat)
            else if (right) {
                if (!last_right || repeat_delay == 0) {
                    if (is_valid_position(board, current_piece, current_rotation, current_x + 1, current_y)) {
                        current_x++;
                        move_delay = 3;
                        repeat_delay = last_right ? 2 : 8;  // Faster repeat after initial delay
                    }
                }
            }
            // Rotate piece clockwise (UP or Z key) with wall kicks
            else if ((up && !last_up) || (z && !last_z)) {
                int next_rotation = (current_rotation + 1) % 4;
                int kick_x = current_x;
                int kick_y = current_y;
                int rotation_success = 0;
                
                // Try rotation at current position
                if (is_valid_position(board, current_piece, next_rotation, kick_x, kick_y)) {
                    rotation_success = 1;
                }
                // Try wall kick left (1 space)
                else if (is_valid_position(board, current_piece, next_rotation, kick_x - 1, kick_y)) {
                    kick_x--;
                    rotation_success = 1;
                }
                // Try wall kick right (1 space)
                else if (is_valid_position(board, current_piece, next_rotation, kick_x + 1, kick_y)) {
                    kick_x++;
                    rotation_success = 1;
                }
                // Try wall kick left (2 spaces) - for I-piece
                else if (is_valid_position(board, current_piece, next_rotation, kick_x - 2, kick_y)) {
                    kick_x -= 2;
                    rotation_success = 1;
                }
                // Try wall kick right (2 spaces) - for I-piece
                else if (is_valid_position(board, current_piece, next_rotation, kick_x + 2, kick_y)) {
                    kick_x += 2;
                    rotation_success = 1;
                }
                // Try kick up (for pieces stuck at bottom)
                else if (is_valid_position(board, current_piece, next_rotation, kick_x, kick_y - 1)) {
                    kick_y--;
                    rotation_success = 1;
                }
                // Try combined kicks
                else if (is_valid_position(board, current_piece, next_rotation, kick_x - 1, kick_y - 1)) {
                    kick_x--;
                    kick_y--;
                    rotation_success = 1;
                }
                else if (is_valid_position(board, current_piece, next_rotation, kick_x + 1, kick_y - 1)) {
                    kick_x++;
                    kick_y--;
                    rotation_success = 1;
                }
                
                if (rotation_success) {
                    current_rotation = next_rotation;
                    current_x = kick_x;
                    current_y = kick_y;
                    move_delay = 5;
                }
            }
            // Instant drop (X key)
            else if (x && !last_x) {
                // Drop piece all the way down
                while (is_valid_position(board, current_piece, current_rotation, current_x, current_y + 1)) {
                    current_y++;
                    score += 2;  // Bonus points for hard drop
                }
                // Force immediate placement
                drop_timer = 100;  // Will trigger placement on next frame
            }
            // Fast drop (DOWN key)
            else if (down) {
                if (is_valid_position(board, current_piece, current_rotation, current_x, current_y + 1)) {
                    current_y++;
                    score++;
                    drop_timer = 0;
                    move_delay = 2;
                }
            }
        }
        
        if (move_delay > 0) move_delay--;
        
        // Update repeat delay
        if (repeat_delay > 0) {
            repeat_delay--;
        }
        
        // Reset repeat delay if key released
        if (!left && !right) {
            repeat_delay = 0;
        }
        
        // Store last key states
        last_left = left;
        last_right = right;
        last_up = up;
        last_down = down;
        last_z = z;
        last_x = x;
        
        // Auto drop
        drop_timer++;
        int drop_speed = 30 - (level * 2);
        if (drop_speed < 5) drop_speed = 5;
        
        if (drop_timer >= drop_speed) {
            if (is_valid_position(board, current_piece, current_rotation, current_x, current_y + 1)) {
                current_y++;
            } else {
                // Piece landed
                place_piece(board, current_piece, current_rotation, current_x, current_y);
                
                // Clear lines
                int cleared = clear_lines(board);
                if (cleared > 0) {
                    lines += cleared;
                    score += cleared * 100 * level;
                    level = 1 + (lines / 10);
                    if (level > 10) level = 10;
                }
                
                // Spawn new piece (use the next_piece)
                current_piece = next_piece;
                next_piece = rng_get() % 7;  // Generate new next piece
                current_x = 3;
                current_y = 0;
                current_rotation = 0;
                
                // Check game over
                if (!is_valid_position(board, current_piece, current_rotation, current_x, current_y)) {
                    game_over = 1;
                }
            }
            drop_timer = 0;
        }
        
        // Draw everything
        draw_ui(score, lines, level);
        draw_next_piece(next_piece);
        draw_board(board);
        draw_ghost_piece(board, current_piece, current_rotation, current_x, current_y);
        draw_current_piece(current_piece, current_rotation, current_x, current_y, get_piece_color(current_piece));
        
        // Flush display
        graphics_flush();
        
        // Frame delay
        for (int d = 0; d < 3000; d++);
    }
    
    // Game over screen
    clear_screen(rgb565(20, 0, 0));
    fill_rect(10, 25, 44, 14, RGB_RED);
    fill_rect(12, 27, 40, 10, RGB_BLACK);
    fill_rect(22, 30, 20, 4, RGB_WHITE);
    graphics_flush();
    
    // Wait
    for (int d = 0; d < 30000; d++);
    
    return 0;
}