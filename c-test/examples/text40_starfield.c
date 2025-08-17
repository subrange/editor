// Classic "flying through space" starfield animation
// Features parallax scrolling with stars at different depths

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Additional color that's not in the standard PICO-8 palette
// Using light blue/cyan approximation
#define COLOR_BLUE COLOR_BLUE

#define MAX_STARS 50
#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25

// Star structure - each star has position and depth
struct Star {
    int x;      // X position (fixed point: actual * 256)
    int y;      // Y position (fixed point: actual * 256)
    int z;      // Depth/speed (1=far/slow, 3=close/fast)
    char active; // Is this star slot active?
};

struct Star stars[MAX_STARS];
int star_count = 0;

// Get random number in range [0, max)
int rand_range(int max) {
    return (rng_get() % max);
}

// Initialize a star at random edge position
void init_star(struct Star* star) {
    // Start stars at edges of screen
    int edge = rand_range(4);
    
    if (edge == 0) { // Top edge
        star->x = rand_range(SCREEN_WIDTH) * 256;
        star->y = 0;
    } else if (edge == 1) { // Bottom edge
        star->x = rand_range(SCREEN_WIDTH) * 256;
        star->y = (SCREEN_HEIGHT - 1) * 256;
    } else if (edge == 2) { // Left edge
        star->x = 0;
        star->y = rand_range(SCREEN_HEIGHT) * 256;
    } else { // Right edge
        star->x = (SCREEN_WIDTH - 1) * 256;
        star->y = rand_range(SCREEN_HEIGHT) * 256;
    }
    
    // Random depth (1-3)
    star->z = 1 + rand_range(3);
    star->active = 1;
}

// Initialize starfield
void init_starfield() {
    for (int i = 0; i < MAX_STARS; i++) {
        if (i < 30) { // Start with 30 stars
            init_star(&stars[i]);
            // Spread them out initially
            stars[i].x = rand_range(SCREEN_WIDTH) * 256;
            stars[i].y = rand_range(SCREEN_HEIGHT) * 256;
            star_count++;
        } else {
            stars[i].active = 0;
        }
    }
}

// Update star positions (flying from center outward)
void update_stars() {
    int center_x = (SCREEN_WIDTH / 2) * 256;
    int center_y = (SCREEN_HEIGHT / 2) * 256;
    
    for (int i = 0; i < MAX_STARS; i++) {
        if (!stars[i].active) continue;
        
        // Calculate direction from center
        int dx = stars[i].x - center_x;
        int dy = stars[i].y - center_y;
        
        // Move away from center based on depth/speed
        stars[i].x += (dx * stars[i].z) / 64;
        stars[i].y += (dy * stars[i].z) / 64;
        
        // Check if star went off screen
        int screen_x = stars[i].x / 256;
        int screen_y = stars[i].y / 256;
        
        if (screen_x < 0 || screen_x >= SCREEN_WIDTH ||
            screen_y < 0 || screen_y >= SCREEN_HEIGHT) {
            // Respawn near center
            stars[i].x = center_x + (rand_range(10) - 5) * 256;
            stars[i].y = center_y + (rand_range(10) - 5) * 256;
            stars[i].z = 1 + rand_range(3);
        }
    }
}

// Draw all stars
void draw_stars() {
    // Clear screen first
    display_clear();
    
    for (int i = 0; i < MAX_STARS; i++) {
        if (!stars[i].active) continue;
        
        int x = stars[i].x / 256;
        int y = stars[i].y / 256;
        
        // Make sure we're in bounds
        if (x >= 0 && x < SCREEN_WIDTH && y >= 0 && y < SCREEN_HEIGHT) {
            // Choose character and color based on depth
            char star_char;
            unsigned char color;
            
            if (stars[i].z == 3) {
                // Close stars - bright and large
                star_char = '*';
                color = COLOR_WHITE;
            } else if (stars[i].z == 2) {
                // Medium distance - dimmer
                star_char = '+';
                color = COLOR_LIGHT_GRAY;
            } else {
                // Far stars - dim and small
                star_char = '.';
                color = COLOR_DARK_GRAY;
            }
            
            text40_putchar_color(x, y, star_char, color, COLOR_BLACK);
        }
    }
}

// Draw a spaceship in the center
void draw_spaceship() {
    int center_x = SCREEN_WIDTH / 2;
    int center_y = SCREEN_HEIGHT / 2;
    
    // Simple spaceship shape
    //    ^
    //   /|\
    //  / | \
    
    text40_putchar_color(center_x, center_y - 1, '^', COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(center_x - 1, center_y, '/', COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(center_x, center_y, '|', COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(center_x + 1, center_y, '\\', COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(center_x - 2, center_y + 1, '/', COLOR_DARK_BLUE, COLOR_BLACK);
    text40_putchar_color(center_x - 1, center_y + 1, ' ', COLOR_BLACK, COLOR_DARK_BLUE);
    text40_putchar_color(center_x, center_y + 1, '|', COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(center_x + 1, center_y + 1, ' ', COLOR_BLACK, COLOR_DARK_BLUE);
    text40_putchar_color(center_x + 2, center_y + 1, '\\', COLOR_DARK_BLUE, COLOR_BLACK);
    
    // Engine glow
    text40_putchar_color(center_x - 1, center_y + 2, ' ', COLOR_BLACK, COLOR_ORANGE);
    text40_putchar_color(center_x, center_y + 2, ' ', COLOR_BLACK, COLOR_RED);
    text40_putchar_color(center_x + 1, center_y + 2, ' ', COLOR_BLACK, COLOR_ORANGE);
}

// Draw HUD elements
void draw_hud() {
    // Top status bar
    text40_puts_color(1, 0, "STARFIELD", COLOR_YELLOW, COLOR_DARK_BLUE);
    text40_puts_color(11, 0, "Speed:", COLOR_WHITE, COLOR_DARK_BLUE);
    text40_puts_color(18, 0, "WARP", COLOR_GREEN, COLOR_DARK_BLUE);
    
    // Bottom status
    text40_puts_color(1, 24, "Sector:", COLOR_LIGHT_GRAY, COLOR_DARK_GRAY);
    text40_puts_color(9, 24, "ALPHA-7", COLOR_BLUE, COLOR_DARK_GRAY);
    text40_puts_color(25, 24, "System: SOL", COLOR_GREEN, COLOR_DARK_GRAY);
    
    // Side decorations
    for (int i = 1; i < 24; i++) {
        if (i % 3 == 0) {
            text40_putchar_color(0, i, '|', COLOR_DARK_GRAY, COLOR_BLACK);
            text40_putchar_color(39, i, '|', COLOR_DARK_GRAY, COLOR_BLACK);
        }
    }
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Add occasional "nebula" or colored space clouds
void draw_nebula(int frame) {
    // Pulsing nebula in corner
    int intensity = (frame / 4) % 4;
    unsigned char nebula_colors[] = {
        COLOR_DARK_PURPLE,
        COLOR_DARK_BLUE,
        COLOR_INDIGO,
        COLOR_DARK_BLUE
    };
    
    // Top-right nebula
    for (int x = 30; x < 39; x++) {
        for (int y = 1; y < 5; y++) {
            if ((x + y + frame/8) % 3 == 0) {
                text40_putchar_color(x, y, ' ', COLOR_BLACK, 
                                   nebula_colors[intensity]);
            }
        }
    }
    
    // Bottom-left nebula
    unsigned char nebula_colors2[] = {
        COLOR_DARK_GREEN,
        COLOR_DARK_GRAY,
        COLOR_DARK_GREEN,
        COLOR_BLACK
    };
    
    for (int x = 1; x < 10; x++) {
        for (int y = 19; y < 23; y++) {
            if ((x + y + frame/8 + 1) % 3 == 0) {
                text40_putchar_color(x, y, ' ', COLOR_BLACK,
                                   nebula_colors2[intensity]);
            }
        }
    }
}

int main() {
    // Enable TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Initialize starfield
    init_starfield();
    
    // Main animation loop
    for (int frame = 0; frame < 200; frame++) {
        // Update physics
        update_stars();
        
        // Draw everything
        draw_stars();
        draw_nebula(frame);
        draw_spaceship();
        draw_hud();
        
        // Add frame counter
        text40_putchar_color(37, 0, '0' + ((frame / 100) % 10), COLOR_WHITE, COLOR_DARK_BLUE);
        text40_putchar_color(38, 0, '0' + ((frame / 10) % 10), COLOR_WHITE, COLOR_DARK_BLUE);
        
        // Speed indicator animation
        int speed_bar = (frame % 8);
        for (int i = 0; i < 5; i++) {
            if (i <= speed_bar / 2) {
                text40_putchar_color(23 + i, 0, '=', COLOR_GREEN, COLOR_DARK_BLUE);
            } else {
                text40_putchar_color(23 + i, 0, '-', COLOR_DARK_GREEN, COLOR_DARK_BLUE);
            }
        }
        
        // Occasional "hyperspace" effect
        if (frame >= 80 && frame < 100) {
            // Add streaking effect
            for (int i = 0; i < MAX_STARS; i++) {
                if (stars[i].active && stars[i].z == 3) {
                    int x = stars[i].x / 256;
                    int y = stars[i].y / 256;
                    int center_x = SCREEN_WIDTH / 2;
                    int center_y = SCREEN_HEIGHT / 2;
                    
                    // Draw streak toward center
                    int dx = (x > center_x) ? 1 : -1;
                    int dy = (y > center_y) ? 1 : -1;
                    
                    if (x - dx >= 0 && x - dx < SCREEN_WIDTH &&
                        y - dy >= 0 && y - dy < SCREEN_HEIGHT) {
                        text40_putchar_color(x - dx, y - dy, '-', COLOR_BLUE, COLOR_BLACK);
                    }
                }
            }
            text40_puts_color(15, 12, "HYPERSPACE", COLOR_YELLOW, COLOR_RED);
        }
        
        display_flush();
        delay(3000); // Animation speed
        
        // Add new stars occasionally
        if (frame % 10 == 0 && star_count < MAX_STARS - 5) {
            for (int i = 0; i < MAX_STARS; i++) {
                if (!stars[i].active) {
                    // Spawn new star near center
                    stars[i].x = (SCREEN_WIDTH/2 + rand_range(6) - 3) * 256;
                    stars[i].y = (SCREEN_HEIGHT/2 + rand_range(6) - 3) * 256;
                    stars[i].z = 1 + rand_range(3);
                    stars[i].active = 1;
                    star_count++;
                    break;
                }
            }
        }
    }
    
    // End sequence
    text40_puts_color(12, 11, "DESTINATION", COLOR_WHITE, COLOR_GREEN);
    text40_puts_color(14, 12, "REACHED", COLOR_BLACK, COLOR_YELLOW);
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Starfield animation complete!");
    
    return 0;
}