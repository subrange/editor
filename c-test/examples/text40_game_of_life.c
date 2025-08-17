// Conway's Game of Life for TEXT40 display
// Uses 1D array indexing since 2D arrays don't work yet

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

#define GRID_WIDTH 38   // Leave 1 char border on each side
#define GRID_HEIGHT 22  // Leave space for title and status
#define GRID_SIZE 836   // 38 * 22 = 836

// Game state - using two buffers for double buffering
unsigned char grid_current[836];
unsigned char grid_next[836];

// Get random number in range [0, max)
int rand_range(int max) {
    return (rng_get() % max);
}

// Convert 2D coordinates to 1D array index
int get_index(int x, int y) {
    return y * GRID_WIDTH + x;
}

// Check if coordinates are valid
int is_valid(int x, int y) {
    return x >= 0 && x < GRID_WIDTH && y >= 0 && y < GRID_HEIGHT;
}

// Count living neighbors for a cell
int count_neighbors(int x, int y) {
    int count = 0;
    
    // Check all 8 neighbors
    for (int dy = -1; dy <= 1; dy++) {
        for (int dx = -1; dx <= 1; dx++) {
            // Skip the center cell
            if (dx == 0 && dy == 0) continue;
            
            int nx = x + dx;
            int ny = y + dy;
            
            // Check bounds and count if alive
            if (is_valid(nx, ny)) {
                if (grid_current[get_index(nx, ny)]) {
                    count++;
                }
            }
        }
    }
    
    return count;
}

// Initialize grid with random pattern
void init_random_pattern() {
    for (int i = 0; i < GRID_SIZE; i++) {
        // About 30% chance of being alive initially
        grid_current[i] = (rand_range(100) < 30) ? 1 : 0;
        grid_next[i] = 0;
    }
}

// Initialize with a glider pattern in the center
void add_glider(int start_x, int start_y) {
    // Classic glider pattern:
    //   X
    //    X
    //  XXX
    
    if (is_valid(start_x + 2, start_y + 2)) {
        grid_current[get_index(start_x + 1, start_y)] = 1;
        grid_current[get_index(start_x + 2, start_y + 1)] = 1;
        grid_current[get_index(start_x, start_y + 2)] = 1;
        grid_current[get_index(start_x + 1, start_y + 2)] = 1;
        grid_current[get_index(start_x + 2, start_y + 2)] = 1;
    }
}

// Add a blinker pattern (oscillator)
void add_blinker(int start_x, int start_y) {
    // Blinker: XXX (horizontal)
    if (is_valid(start_x + 2, start_y)) {
        grid_current[get_index(start_x, start_y)] = 1;
        grid_current[get_index(start_x + 1, start_y)] = 1;
        grid_current[get_index(start_x + 2, start_y)] = 1;
    }
}

// Add a block pattern (still life)
void add_block(int start_x, int start_y) {
    // Block: XX
    //        XX
    if (is_valid(start_x + 1, start_y + 1)) {
        grid_current[get_index(start_x, start_y)] = 1;
        grid_current[get_index(start_x + 1, start_y)] = 1;
        grid_current[get_index(start_x, start_y + 1)] = 1;
        grid_current[get_index(start_x + 1, start_y + 1)] = 1;
    }
}

// Initialize with some known patterns
void init_pattern_showcase() {
    // Clear grid first
    for (int i = 0; i < GRID_SIZE; i++) {
        grid_current[i] = 0;
        grid_next[i] = 0;
    }
    
    // Add various patterns
    add_glider(5, 5);
    add_glider(20, 3);
    add_blinker(10, 10);
    add_blinker(25, 15);
    add_block(15, 8);
    add_block(30, 18);
    
    // Add a random cluster in the middle
    int center_x = GRID_WIDTH / 2;
    int center_y = GRID_HEIGHT / 2;
    for (int dy = -3; dy <= 3; dy++) {
        for (int dx = -3; dx <= 3; dx++) {
            int x = center_x + dx;
            int y = center_y + dy;
            if (is_valid(x, y) && rand_range(100) < 40) {
                grid_current[get_index(x, y)] = 1;
            }
        }
    }
}

// Update the grid according to Game of Life rules
void update_grid() {
    // Apply Conway's Game of Life rules
    for (int y = 0; y < GRID_HEIGHT; y++) {
        for (int x = 0; x < GRID_WIDTH; x++) {
            int idx = get_index(x, y);
            int neighbors = count_neighbors(x, y);
            int alive = grid_current[idx];
            
            // Apply the rules:
            // 1. Any live cell with 2-3 neighbors survives
            // 2. Any dead cell with exactly 3 neighbors becomes alive
            // 3. All other cells die or stay dead
            
            if (alive) {
                if (neighbors == 2 || neighbors == 3) {
                    grid_next[idx] = 1;
                } else {
                    grid_next[idx] = 0;
                }
            } else {
                if (neighbors == 3) {
                    grid_next[idx] = 1;
                } else {
                    grid_next[idx] = 0;
                }
            }
        }
    }
    
    // Swap buffers
    for (int i = 0; i < GRID_SIZE; i++) {
        grid_current[i] = grid_next[i];
    }
}

// Count total living cells
int count_alive() {
    int count = 0;
    for (int i = 0; i < GRID_SIZE; i++) {
        if (grid_current[i]) count++;
    }
    return count;
}

// Draw the grid to the display
void draw_grid() {
    // Draw cells
    for (int y = 0; y < GRID_HEIGHT; y++) {
        for (int x = 0; x < GRID_WIDTH; x++) {
            int idx = get_index(x, y);
            char cell_char;
            unsigned char fg_color, bg_color;
            
            if (grid_current[idx]) {
                // Living cell - use different colors based on position for visual interest
                cell_char = ' ';
                
                // Create a gradient effect
                int color_idx = ((x + y) / 8) % 4;
                if (color_idx == 0) {
                    fg_color = COLOR_GREEN;
                    bg_color = COLOR_DARK_GREEN;
                } else if (color_idx == 1) {
                    fg_color = COLOR_BLUE;
                    bg_color = COLOR_DARK_BLUE;
                } else if (color_idx == 2) {
                    fg_color = COLOR_PINK;
                    bg_color = COLOR_DARK_PURPLE;
                } else {
                    fg_color = COLOR_YELLOW;
                    bg_color = COLOR_BROWN;
                }
            } else {
                // Dead cell
                cell_char = '.';
                fg_color = COLOR_DARK_GRAY;
                bg_color = COLOR_BLACK;
            }
            
            text40_putchar_color(x + 1, y + 2, cell_char, fg_color, bg_color);
        }
    }
}

// Draw the UI frame and status
void draw_ui(int generation, int population) {
    // Title bar
    text40_puts_color(10, 0, " GAME OF LIFE ", COLOR_WHITE, COLOR_DARK_BLUE);
    
    // Draw border
    for (int x = 0; x < 40; x++) {
        text40_putchar_color(x, 1, '=', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(x, 24, '=', COLOR_BLUE, COLOR_BLACK);
    }
    
    for (int y = 2; y < 24; y++) {
        text40_putchar_color(0, y, '|', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(39, y, '|', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Generation counter
    text40_puts_color(1, 0, "Gen:", COLOR_LIGHT_GRAY, COLOR_BLACK);
    text40_putchar_color(5, 0, '0' + ((generation / 100) % 10), COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(6, 0, '0' + ((generation / 10) % 10), COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(7, 0, '0' + (generation % 10), COLOR_WHITE, COLOR_BLACK);
    
    // Population counter
    text40_puts_color(26, 0, "Pop:", COLOR_LIGHT_GRAY, COLOR_BLACK);
    text40_putchar_color(30, 0, '0' + ((population / 1000) % 10), COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(31, 0, '0' + ((population / 100) % 10), COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(32, 0, '0' + ((population / 10) % 10), COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(33, 0, '0' + (population % 10), COLOR_WHITE, COLOR_BLACK);
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
    
    // Initialize with pattern showcase
    init_pattern_showcase();
    
    // Main game loop
    int generation = 0;
    int stable_count = 0;
    int last_population = 0;
    
    for (generation = 0; generation < 300; generation++) {
        // Clear display
        display_clear();
        
        // Count living cells
        int population = count_alive();
        
        // Draw everything
        draw_ui(generation, population);
        draw_grid();
        
        // Check for extinction or stability
        if (population == 0) {
            text40_puts_color(12, 12, " EXTINCTION! ", COLOR_WHITE, COLOR_RED);
            display_flush();
            delay(10000);
            break;
        }
        
        // Check if population is stable
        if (population == last_population) {
            stable_count++;
            if (stable_count > 10) {
                text40_puts_color(11, 12, " STABLE STATE ", COLOR_BLACK, COLOR_GREEN);
            }
        } else {
            stable_count = 0;
        }
        last_population = population;
        
        // Update display
        display_flush();
        
        // Animation speed
        delay(5000);
        
        // Update grid for next generation
        update_grid();
        
        // Every 50 generations, add some random cells to keep it interesting
        if (generation > 0 && generation % 50 == 0) {
            // Add a few random cells
            for (int i = 0; i < 20; i++) {
                int x = rand_range(GRID_WIDTH);
                int y = rand_range(GRID_HEIGHT);
                grid_current[get_index(x, y)] = 1;
            }
            
            // Flash message
            text40_puts_color(10, 12, " MUTATION! ", COLOR_YELLOW, COLOR_RED);
            display_flush();
            delay(10000);
        }
    }
    
    // Final message
    text40_puts_color(8, 11, " SIMULATION ", COLOR_WHITE, COLOR_DARK_PURPLE);
    text40_puts_color(11, 12, " COMPLETE ", COLOR_BLACK, COLOR_YELLOW);
    display_flush();
    delay(10000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Game of Life simulation complete!");
    
    return 0;
}