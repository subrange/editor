// 3D Labyrinth Demo - Windows 95 Style Screensaver
// Navigate through a 3D maze with perspective rendering

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25
#define VIEW_DISTANCE 5
#define MAZE_SIZE 16

// Maze cell types
#define CELL_EMPTY 0
#define CELL_WALL 1

// Direction constants
#define DIR_NORTH 0
#define DIR_EAST 1
#define DIR_SOUTH 2
#define DIR_WEST 3

// Simple maze layout (16x16) - using 1D array
// Using literal 256 (16*16) since compiler doesn't support macro expressions in array sizes
unsigned char maze[256];

// Player position and direction
int player_x = 1;
int player_y = 1;
int player_dir = DIR_NORTH;

// Animation frame counter
int anim_frame = 0;

// Get random number in range [0, max)
int rand_range(int max) {
    return (rng_get() % max);
}

// Helper to access maze array
unsigned char get_maze(int x, int y) {
    if (x < 0 || x >= MAZE_SIZE || y < 0 || y >= MAZE_SIZE) {
        return CELL_WALL;
    }
    return maze[y * MAZE_SIZE + x];
}

void set_maze(int x, int y, unsigned char value) {
    if (x >= 0 && x < MAZE_SIZE && y >= 0 && y < MAZE_SIZE) {
        maze[y * MAZE_SIZE + x] = value;
    }
}

// Initialize a simple maze
void init_maze() {
    // Fill everything with walls first
    for (int i = 0; i < 256; i++) {
        maze[i] = CELL_WALL;
    }
    
    // Create a more structured maze with guaranteed outer walls
    // Carve horizontal corridors
    for (int y = 2; y < MAZE_SIZE - 2; y += 3) {
        for (int x = 1; x < MAZE_SIZE - 1; x++) {
            if (x > 0 && x < MAZE_SIZE - 1 && y > 0 && y < MAZE_SIZE - 1) {
                set_maze(x, y, CELL_EMPTY);
            }
        }
    }
    
    // Carve vertical corridors  
    for (int x = 2; x < MAZE_SIZE - 2; x += 3) {
        for (int y = 1; y < MAZE_SIZE - 1; y++) {
            if (x > 0 && x < MAZE_SIZE - 1 && y > 0 && y < MAZE_SIZE - 1) {
                set_maze(x, y, CELL_EMPTY);
            }
        }
    }
    
    // Add some walls back to create maze structure
    for (int i = 0; i < 30; i++) {
        int x = 1 + rand_range(MAZE_SIZE - 2);
        int y = 1 + rand_range(MAZE_SIZE - 2);
        // Don't block the starting area
        if (x > 3 || y > 3) {
            set_maze(x, y, CELL_WALL);
        }
    }
    
    // Clear starting area
    set_maze(1, 1, CELL_EMPTY);
    set_maze(2, 1, CELL_EMPTY);
    set_maze(1, 2, CELL_EMPTY);
    set_maze(2, 2, CELL_EMPTY);
    
    // Force outer walls to be solid
    for (int i = 0; i < MAZE_SIZE; i++) {
        set_maze(i, 0, CELL_WALL);              // Top wall
        set_maze(i, MAZE_SIZE - 1, CELL_WALL);  // Bottom wall
        set_maze(0, i, CELL_WALL);              // Left wall
        set_maze(MAZE_SIZE - 1, i, CELL_WALL);  // Right wall
    }
    
    // Update player start position  
    player_x = 1;
    player_y = 1;
}

// Get cell in front of player at given distance
int get_cell_ahead(int distance, int offset) {
    int dx = 0;
    int dy = 0;
    
    // Calculate forward direction
    if (player_dir == DIR_NORTH) {
        dy = -1;
    } else if (player_dir == DIR_SOUTH) {
        dy = 1;
    } else if (player_dir == DIR_EAST) {
        dx = 1;
    } else if (player_dir == DIR_WEST) {
        dx = -1;
    }
    
    // Calculate perpendicular offset
    int perp_dx = 0;
    int perp_dy = 0;
    if (player_dir == DIR_NORTH || player_dir == DIR_SOUTH) {
        perp_dx = offset;
    } else {
        perp_dy = offset;
    }
    
    // Calculate final position
    int check_x = player_x + dx * distance + perp_dx;
    int check_y = player_y + dy * distance + perp_dy;
    
    // Bounds check - treat out of bounds as wall
    if (check_x < 0 || check_x >= MAZE_SIZE || 
        check_y < 0 || check_y >= MAZE_SIZE) {
        return CELL_WALL;
    }
    
    return get_maze(check_x, check_y);
}

// Get actual distance to nearest wall in front
int get_wall_distance() {
    for (int d = 0; d <= VIEW_DISTANCE; d++) {
        if (get_cell_ahead(d, 0) == CELL_WALL) {
            return d;
        }
    }
    return VIEW_DISTANCE + 1;
}

// Draw perspective view of the corridor
void draw_3d_view() {
    // Clear screen
    display_clear();
    
    // Get actual wall distance for better rendering
    int front_wall_dist = get_wall_distance();
    
    // Special handling for very close walls
    if (front_wall_dist <= 1) {
        // Fill screen with close wall texture
        for (int y = 0; y < SCREEN_HEIGHT; y++) {
            for (int x = 0; x < SCREEN_WIDTH; x++) {
                // Brick pattern for close wall
                char pattern = ' ';
                unsigned char fg = COLOR_WHITE;
                unsigned char bg = COLOR_LIGHT_GRAY;
                
                // Create brick pattern
                if (y % 4 == 0) {
                    // Horizontal mortar lines
                    bg = COLOR_DARK_GRAY;
                    pattern = '-';
                    fg = COLOR_DARK_GRAY;
                } else if ((x + (y/4)*8) % 16 == 0) {
                    // Vertical mortar lines (offset every 4 rows)
                    bg = COLOR_DARK_GRAY;
                    pattern = '|';
                    fg = COLOR_DARK_GRAY;
                } else if ((x + y) % 7 == 0) {
                    // Some texture on the bricks
                    pattern = '.';
                    fg = COLOR_DARK_GRAY;
                }
                
                if (pattern != ' ') {
                    text40_putchar_color(x, y, pattern, fg, bg);
                } else {
                    text40_putchar_color(x, y, ' ', COLOR_BLACK, bg);
                }
            }
        }
        return; // Don't draw anything else
    }
    
    // Draw walls from far to near for proper overlap
    for (int dist = VIEW_DISTANCE; dist >= 0; dist--) {
        // Calculate perspective scaling
        int wall_height = (SCREEN_HEIGHT - 2) / (dist + 1);
        int wall_top = (SCREEN_HEIGHT - wall_height) / 2;
        int wall_bottom = wall_top + wall_height;
        
        int wall_width = (SCREEN_WIDTH - 4) / (dist + 1);
        int wall_left = (SCREEN_WIDTH - wall_width) / 2;
        int wall_right = wall_left + wall_width;
        
        // Check walls at this distance
        int left_wall = get_cell_ahead(dist, -1);
        int center_wall = get_cell_ahead(dist, 0);
        int right_wall = get_cell_ahead(dist, 1);
        
        // Choose wall colors based on distance
        unsigned char wall_color = COLOR_DARK_GRAY;
        unsigned char edge_color = COLOR_LIGHT_GRAY;
        if (dist <= 1) {
            wall_color = COLOR_LIGHT_GRAY;
            edge_color = COLOR_WHITE;
        } else if (dist == 2) {
            wall_color = COLOR_DARK_GRAY;
            edge_color = COLOR_LIGHT_GRAY;
        } else if (dist == 3) {
            wall_color = COLOR_DARK_GRAY;
            edge_color = COLOR_DARK_GRAY;
        } else {
            // Far walls - make them more visible
            wall_color = COLOR_DARK_BLUE;
            edge_color = COLOR_BLUE;
        }
        
        // Draw left wall
        if (left_wall == CELL_WALL) {
            for (int y = wall_top; y < wall_bottom; y++) {
                for (int x = 0; x < wall_left; x++) {
                    // Make walls more solid, especially at distance 2-3
                    if (dist <= 3 && x >= wall_left - 2) {
                        // Near edge - draw solid
                        text40_putchar_color(x, y, ' ', COLOR_BLACK, wall_color);
                    } else if (x == wall_left - 1 || y == wall_top || y == wall_bottom - 1) {
                        text40_putchar_color(x, y, '#', edge_color, COLOR_BLACK);
                    } else if (x < wall_left - 1) {
                        text40_putchar_color(x, y, ' ', COLOR_BLACK, wall_color);
                    }
                }
            }
        }
        
        // Draw right wall
        if (right_wall == CELL_WALL) {
            for (int y = wall_top; y < wall_bottom; y++) {
                for (int x = wall_right; x < SCREEN_WIDTH; x++) {
                    // Make walls more solid, especially at distance 2-3
                    if (dist <= 3 && x <= wall_right + 1) {
                        // Near edge - draw solid
                        text40_putchar_color(x, y, ' ', COLOR_BLACK, wall_color);
                    } else if (x == wall_right || y == wall_top || y == wall_bottom - 1) {
                        text40_putchar_color(x, y, '#', edge_color, COLOR_BLACK);
                    } else if (x > wall_right) {
                        text40_putchar_color(x, y, ' ', COLOR_BLACK, wall_color);
                    }
                }
            }
        }
        
        // Draw center/back wall
        if (center_wall == CELL_WALL) {
            // Special case for wall very close (distance 0 or 1) - RED FOR TESTING
            if (dist == 0 || dist == 1) {
                // Fill entire/most of screen with wall - TEMPORARILY RED FOR TESTING
                for (int y = 0; y < SCREEN_HEIGHT; y++) {
                    for (int x = 0; x < SCREEN_WIDTH; x++) {
                        // Add brick pattern for very close wall
                        char pattern = ' ';
                        unsigned char bg = COLOR_RED;
                        if ((y % 3 == 0) || ((x + (y/3)*5) % 8 == 0)) {
                            bg = COLOR_RED;
                            pattern = '#';
                        }
                        if (pattern == '#') {
                            text40_putchar_color(x, y, pattern, COLOR_YELLOW, bg);
                        } else {
                            text40_putchar_color(x, y, ' ', COLOR_BLACK, bg);
                        }
                    }
                }
            } else {
                // Normal wall rendering for other distances
                for (int y = wall_top; y < wall_bottom; y++) {
                    for (int x = wall_left; x < wall_right; x++) {
                        // For distance 2, make the wall more solid
                        if (dist == 2) {
                            // Fill the wall solidly with color
                            text40_putchar_color(x, y, ' ', COLOR_BLACK, wall_color);
                            // Add some brick pattern on top
                            if ((x % 3 == 0 || y == wall_top || y == wall_bottom - 1)) {
                                text40_putchar_color(x, y, '#', edge_color, wall_color);
                            }
                        } else if (x == wall_left || x == wall_right - 1 || 
                            y == wall_top || y == wall_bottom - 1) {
                            text40_putchar_color(x, y, '#', edge_color, COLOR_BLACK);
                        } else {
                            // Add some texture pattern for other distances
                            char pattern = ' ';
                            if ((x + y) % 4 == 0 && dist <= 3) {
                                pattern = '.';
                            }
                            if (pattern == ' ') {
                                text40_putchar_color(x, y, ' ', COLOR_BLACK, wall_color);
                            } else {
                                text40_putchar_color(x, y, pattern, edge_color, wall_color);
                            }
                        }
                    }
                }
            }
            break; // Don't draw beyond a wall
        }
        
        // Draw floor pattern
        if (dist <= 3 && center_wall == CELL_EMPTY) {
            int floor_y = wall_bottom;
            if (floor_y < SCREEN_HEIGHT - 1) {
                for (int x = wall_left; x < wall_right; x++) {
                    if ((x + dist) % 3 == 0) {
                        text40_putchar_color(x, floor_y, '-', COLOR_DARK_GRAY, COLOR_BLACK);
                    }
                }
            }
            
            // Draw ceiling pattern
            int ceiling_y = wall_top - 1;
            if (ceiling_y >= 0) {
                for (int x = wall_left; x < wall_right; x++) {
                    if ((x + dist) % 3 == 1) {
                        text40_putchar_color(x, ceiling_y, '.', COLOR_DARK_GRAY, COLOR_BLACK);
                    }
                }
            }
            
            // Draw side wall edges when there's no side passage
            // This creates the corridor effect
            if (dist <= 2) {
                // Left edge line if there's a wall on the left
                if (left_wall == CELL_WALL && center_wall == CELL_EMPTY) {
                    for (int y = wall_top; y < wall_bottom; y++) {
                        if (wall_left > 0) {
                            text40_putchar_color(wall_left - 1, y, '|', COLOR_DARK_GRAY, COLOR_BLACK);
                        }
                    }
                }
                
                // Right edge line if there's a wall on the right  
                if (right_wall == CELL_WALL && center_wall == CELL_EMPTY) {
                    for (int y = wall_top; y < wall_bottom; y++) {
                        if (wall_right < SCREEN_WIDTH) {
                            text40_putchar_color(wall_right, y, '|', COLOR_DARK_GRAY, COLOR_BLACK);
                        }
                    }
                }
            }
        }
    }
    
    // Draw immediate side walls (at player position) for corridor effect
    int immediate_left = get_cell_ahead(0, -1);
    int immediate_right = get_cell_ahead(0, 1);
    
    if (immediate_left == CELL_WALL) {
        // Draw left wall edge very close
        for (int y = 2; y < SCREEN_HEIGHT - 2; y++) {
            text40_putchar_color(0, y, '|', COLOR_LIGHT_GRAY, COLOR_DARK_GRAY);
            text40_putchar_color(1, y, '|', COLOR_DARK_GRAY, COLOR_BLACK);
        }
    }
    
    if (immediate_right == CELL_WALL) {
        // Draw right wall edge very close
        for (int y = 2; y < SCREEN_HEIGHT - 2; y++) {
            text40_putchar_color(SCREEN_WIDTH - 1, y, '|', COLOR_LIGHT_GRAY, COLOR_DARK_GRAY);
            text40_putchar_color(SCREEN_WIDTH - 2, y, '|', COLOR_DARK_GRAY, COLOR_BLACK);
        }
    }
    
    // Draw perspective lines for depth (corners)
    text40_putchar_color(0, 0, '\\', COLOR_DARK_BLUE, COLOR_BLACK);
    text40_putchar_color(SCREEN_WIDTH - 1, 0, '/', COLOR_DARK_BLUE, COLOR_BLACK);
    text40_putchar_color(0, SCREEN_HEIGHT - 1, '/', COLOR_DARK_BLUE, COLOR_BLACK);
    text40_putchar_color(SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, '\\', COLOR_DARK_BLUE, COLOR_BLACK);
}

// Draw mini-map
void draw_minimap() {
    int map_x = SCREEN_WIDTH - 10;
    int map_y = 1;
    
    // Draw 9x9 area around player
    for (int dy = -4; dy <= 4; dy++) {
        for (int dx = -4; dx <= 4; dx++) {
            int mx = player_x + dx;
            int my = player_y + dy;
            
            if (mx >= 0 && mx < MAZE_SIZE && my >= 0 && my < MAZE_SIZE) {
                char c = ' ';
                unsigned char fg = COLOR_BLACK;
                unsigned char bg = COLOR_BLACK;
                
                if (dx == 0 && dy == 0) {
                    // Player position
                    if (player_dir == DIR_NORTH) c = '^';
                    else if (player_dir == DIR_SOUTH) c = 'v';
                    else if (player_dir == DIR_EAST) c = '>';
                    else if (player_dir == DIR_WEST) c = '<';
                    fg = COLOR_YELLOW;
                    bg = COLOR_DARK_GREEN;
                } else if (get_maze(mx, my) == CELL_WALL) {
                    bg = COLOR_DARK_GRAY;
                } else {
                    bg = COLOR_DARK_GREEN;
                }
                
                text40_putchar_color(map_x + dx + 4, map_y + dy + 4, c, fg, bg);
            }
        }
    }
}

// Draw HUD
void draw_hud() {
    // Title
    text40_puts_color(1, 0, "3D LABYRINTH", COLOR_YELLOW, COLOR_DARK_BLUE);
    
    // Direction indicator
    char* dir_str = "N";
    if (player_dir == DIR_EAST) dir_str = "E";
    else if (player_dir == DIR_SOUTH) dir_str = "S";
    else if (player_dir == DIR_WEST) dir_str = "W";
    
    text40_puts_color(1, SCREEN_HEIGHT - 1, "DIR:", COLOR_WHITE, COLOR_DARK_GRAY);
    text40_puts_color(5, SCREEN_HEIGHT - 1, dir_str, COLOR_GREEN, COLOR_DARK_GRAY);
    
    // Position
    text40_puts_color(10, SCREEN_HEIGHT - 1, "POS:", COLOR_WHITE, COLOR_DARK_GRAY);
    text40_putchar_color(14, SCREEN_HEIGHT - 1, '0' + (player_x / 10), COLOR_BLUE, COLOR_DARK_GRAY);
    text40_putchar_color(15, SCREEN_HEIGHT - 1, '0' + (player_x % 10), COLOR_BLUE, COLOR_DARK_GRAY);
    text40_putchar_color(16, SCREEN_HEIGHT - 1, ',', COLOR_WHITE, COLOR_DARK_GRAY);
    text40_putchar_color(17, SCREEN_HEIGHT - 1, '0' + (player_y / 10), COLOR_BLUE, COLOR_DARK_GRAY);
    text40_putchar_color(18, SCREEN_HEIGHT - 1, '0' + (player_y % 10), COLOR_BLUE, COLOR_DARK_GRAY);
}

// Move player forward if possible
void move_forward() {
    int new_x = player_x;
    int new_y = player_y;
    
    if (player_dir == DIR_NORTH) new_y--;
    else if (player_dir == DIR_SOUTH) new_y++;
    else if (player_dir == DIR_EAST) new_x++;
    else if (player_dir == DIR_WEST) new_x--;
    
    // Check bounds and collision
    if (new_x >= 0 && new_x < MAZE_SIZE && 
        new_y >= 0 && new_y < MAZE_SIZE &&
        get_maze(new_x, new_y) == CELL_EMPTY) {
        player_x = new_x;
        player_y = new_y;
    }
}

// Turn player
void turn_left() {
    player_dir = (player_dir + 3) % 4;
}

void turn_right() {
    player_dir = (player_dir + 1) % 4;
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Track last few moves to avoid getting stuck
int moves_since_turn = 0;
int last_direction_change = 0;

// Automatic navigation AI - improved wall-following
void auto_navigate() {
    // Check what's around us
    int front_clear = (get_cell_ahead(1, 0) == CELL_EMPTY);
    int front_far_clear = (get_cell_ahead(2, 0) == CELL_EMPTY);
    // Check to the sides (offset perpendicular to current direction)
    int left_clear = (get_cell_ahead(0, -1) == CELL_EMPTY);
    int right_clear = (get_cell_ahead(0, 1) == CELL_EMPTY);
    
    moves_since_turn++;
    
    // Simple but effective navigation
    if (!front_clear) {
        // Wall directly ahead - must turn
        // Prefer turning right (right-hand wall following)
        if (rand_range(10) < 7) {
            turn_right();
        } else {
            turn_left();
        }
        moves_since_turn = 0;
    } else {
        // Path ahead is clear
        if (moves_since_turn > 3 && right_clear && rand_range(10) < 3) {
            // Sometimes turn right at intersections
            turn_right();
            moves_since_turn = 0;
        } else if (moves_since_turn > 3 && left_clear && rand_range(10) < 2) {
            // Less frequently turn left
            turn_left();
            moves_since_turn = 0;
        } else if (front_clear) {
            // Keep moving forward
            move_forward();
            
            // But don't get stuck in long corridors - occasionally turn
            if (moves_since_turn > 8 && rand_range(10) < 3) {
                if (rand_range(2) == 0) {
                    turn_right();
                } else {
                    turn_left();
                }
                moves_since_turn = 0;
            }
        }
    }
}

int main() {
    // Enable TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Initialize maze
    init_maze();
    
    // Main animation loop
    for (int frame = 0; frame < 300; frame++) {
        // Draw 3D view
        draw_3d_view();
        
        // Draw mini-map
        draw_minimap();
        
        // Draw HUD
        draw_hud();
        
        // Frame counter
        text40_putchar_color(SCREEN_WIDTH - 3, 0, '0' + ((frame / 100) % 10), COLOR_WHITE, COLOR_DARK_BLUE);
        text40_putchar_color(SCREEN_WIDTH - 2, 0, '0' + ((frame / 10) % 10), COLOR_WHITE, COLOR_DARK_BLUE);
        text40_putchar_color(SCREEN_WIDTH - 1, 0, '0' + (frame % 10), COLOR_WHITE, COLOR_DARK_BLUE);
        
        display_flush();
        delay(8000); // Animation speed - slower for smoother movement
        
        // Navigate automatically - less frequent for calmer movement
        if (frame % 5 == 0) {
            auto_navigate();
        }
        
        anim_frame++;
    }
    
    // End sequence
    display_clear();
    text40_puts_color(12, 11, "MAZE COMPLETE", COLOR_WHITE, COLOR_GREEN);
    text40_puts_color(13, 12, "EXITING...", COLOR_BLACK, COLOR_YELLOW);
    display_flush();
    delay(5000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("3D Labyrinth demo complete!");
    
    return 0;
}