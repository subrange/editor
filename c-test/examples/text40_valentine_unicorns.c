// Valentine's Day Unicorn Paradise - Romantic Fantasy Animation
// Features unicorns, rainbows, hearts, sparkles, and love magic

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25
#define MAX_HEARTS 20
#define MAX_SPARKLES 30
#define MAX_BUTTERFLIES 10
#define MAX_FLOWERS 12
#define MAX_CUPID_ARROWS 5

// Heart structure
struct Heart {
    int x;
    int y;
    int vx;
    int vy;
    int size; // 0=small, 1=medium, 2=large
    unsigned char color;
    char active;
};

// Sparkle structure
struct Sparkle {
    int x;
    int y;
    int lifetime;
    int type; // 0=star, 1=plus, 2=dot
    unsigned char color;
    char active;
};

// Butterfly structure
struct Butterfly {
    int x;
    int y;
    int vx;
    int vy;
    int wing_state;
    char active;
};

// Flower structure
struct Flower {
    int x;
    int y;
    int bloom_state;
    unsigned char color;
    char active;
};

// Cupid arrow structure
struct CupidArrow {
    int x;
    int y;
    int vx;
    int target_x;
    int state; // 0=flying, 1=hit
    char active;
};

// Global arrays
struct Heart hearts[MAX_HEARTS];
struct Sparkle sparkles[MAX_SPARKLES];
struct Butterfly butterflies[MAX_BUTTERFLIES];
struct Flower flowers[MAX_FLOWERS];
struct CupidArrow arrows[MAX_CUPID_ARROWS];

// Unicorn animation state
int unicorn1_x = 8;
int unicorn1_y = 15;
int unicorn2_x = 28;
int unicorn2_y = 15;
int unicorn1_frame = 0;
int unicorn2_frame = 0;
int rainbow_offset = 0;

// Get random number in range [0, max)
int rand_range(int max) {
    return (rng_get() % max);
}

// Initialize hearts
void init_hearts() {
    for (int i = 0; i < MAX_HEARTS; i++) {
        hearts[i].x = rand_range(SCREEN_WIDTH);
        hearts[i].y = rand_range(SCREEN_HEIGHT - 5);
        hearts[i].vx = (rand_range(3) - 1);
        hearts[i].vy = -1;
        hearts[i].size = rand_range(3);
        
        // Valentine colors
        unsigned char valentine_colors[] = {
            COLOR_RED, COLOR_PINK, COLOR_WHITE, COLOR_PEACH
        };
        hearts[i].color = valentine_colors[rand_range(4)];
        hearts[i].active = (i < 10) ? 1 : 0;
    }
}

// Initialize sparkles
void init_sparkles() {
    for (int i = 0; i < MAX_SPARKLES; i++) {
        sparkles[i].x = rand_range(SCREEN_WIDTH);
        sparkles[i].y = rand_range(SCREEN_HEIGHT);
        sparkles[i].lifetime = rand_range(30) + 10;
        sparkles[i].type = rand_range(3);
        
        unsigned char sparkle_colors[] = {
            COLOR_WHITE, COLOR_YELLOW, COLOR_PINK, COLOR_LIGHT_GRAY
        };
        sparkles[i].color = sparkle_colors[rand_range(4)];
        sparkles[i].active = (i < 15) ? 1 : 0;
    }
}

// Initialize butterflies
void init_butterflies() {
    for (int i = 0; i < MAX_BUTTERFLIES; i++) {
        butterflies[i].x = rand_range(SCREEN_WIDTH);
        butterflies[i].y = 2 + rand_range(8);
        butterflies[i].vx = (rand_range(2) == 0) ? 1 : -1;
        butterflies[i].vy = 0;
        butterflies[i].wing_state = rand_range(2);
        butterflies[i].active = (i < 5) ? 1 : 0;
    }
}

// Initialize flowers
void init_flowers() {
    // Place flowers in meadow
    int spacing = SCREEN_WIDTH / MAX_FLOWERS;
    for (int i = 0; i < MAX_FLOWERS; i++) {
        flowers[i].x = (i * spacing) + rand_range(spacing);
        flowers[i].y = 19 + rand_range(3);
        flowers[i].bloom_state = rand_range(3);
        
        unsigned char flower_colors[] = {
            COLOR_RED, COLOR_PINK, COLOR_YELLOW, COLOR_PEACH, COLOR_WHITE
        };
        flowers[i].color = flower_colors[rand_range(5)];
        flowers[i].active = 1;
    }
}

// Update hearts
void update_hearts() {
    for (int i = 0; i < MAX_HEARTS; i++) {
        if (!hearts[i].active) continue;
        
        // Float upward with slight sway
        hearts[i].y += hearts[i].vy;
        hearts[i].x += hearts[i].vx;
        
        // Sway side to side
        if (rand_range(10) < 3) {
            hearts[i].vx = rand_range(3) - 1;
        }
        
        // Reset when off screen
        if (hearts[i].y < 0) {
            hearts[i].y = SCREEN_HEIGHT - 1;
            hearts[i].x = rand_range(SCREEN_WIDTH);
            hearts[i].vx = rand_range(3) - 1;
        }
        
        // Wrap horizontally
        if (hearts[i].x < 0) hearts[i].x = SCREEN_WIDTH - 1;
        if (hearts[i].x >= SCREEN_WIDTH) hearts[i].x = 0;
    }
}

// Update sparkles
void update_sparkles() {
    for (int i = 0; i < MAX_SPARKLES; i++) {
        if (!sparkles[i].active) continue;
        
        sparkles[i].lifetime--;
        
        // Respawn when expired
        if (sparkles[i].lifetime <= 0) {
            sparkles[i].x = rand_range(SCREEN_WIDTH);
            sparkles[i].y = rand_range(SCREEN_HEIGHT);
            sparkles[i].lifetime = rand_range(30) + 10;
            sparkles[i].type = rand_range(3);
        }
    }
}

// Update butterflies
void update_butterflies() {
    for (int i = 0; i < MAX_BUTTERFLIES; i++) {
        if (!butterflies[i].active) continue;
        
        // Flap wings
        butterflies[i].wing_state = (butterflies[i].wing_state + 1) % 2;
        
        // Move in gentle pattern
        butterflies[i].x += butterflies[i].vx;
        
        // Sine wave vertical movement
        if (rand_range(10) < 3) {
            butterflies[i].vy = rand_range(3) - 1;
            butterflies[i].y += butterflies[i].vy;
        }
        
        // Bounce off edges
        if (butterflies[i].x <= 0 || butterflies[i].x >= SCREEN_WIDTH - 1) {
            butterflies[i].vx = -butterflies[i].vx;
        }
        
        // Keep in upper area
        if (butterflies[i].y < 2) butterflies[i].y = 2;
        if (butterflies[i].y > 10) butterflies[i].y = 10;
    }
}

// Spawn new sparkle at position
void spawn_sparkle(int x, int y, unsigned char color) {
    for (int i = 0; i < MAX_SPARKLES; i++) {
        if (!sparkles[i].active) {
            sparkles[i].x = x;
            sparkles[i].y = y;
            sparkles[i].lifetime = 20;
            sparkles[i].type = rand_range(3);
            sparkles[i].color = color;
            sparkles[i].active = 1;
            break;
        }
    }
}

// Draw sky with gradient
void draw_sky() {
    // Pink to purple gradient for Valentine's theme
    for (int y = 0; y < 12; y++) {
        unsigned char sky_color;
        if (y < 4) {
            sky_color = COLOR_PINK;
        } else if (y < 8) {
            sky_color = COLOR_PEACH;
        } else {
            sky_color = COLOR_LIGHT_GRAY;
        }
        
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            text40_putchar_color(x, y, ' ', COLOR_BLACK, sky_color);
        }
    }
    
    // Clouds
    text40_puts_color(5, 2, "~~~", COLOR_WHITE, COLOR_PINK);
    text40_puts_color(25, 3, "~~~~", COLOR_WHITE, COLOR_PEACH);
    text40_puts_color(15, 1, "~~", COLOR_WHITE, COLOR_PINK);
}

// Draw rainbow
void draw_rainbow(int frame) {
    int rainbow_y = 5;
    unsigned char rainbow_colors[] = {
        COLOR_RED, COLOR_ORANGE, COLOR_YELLOW, 
        COLOR_GREEN, COLOR_BLUE, COLOR_INDIGO, COLOR_PINK
    };
    
    // Animated rainbow arc
    for (int arc = 0; arc < 3; arc++) {
        int y = rainbow_y + arc;
        unsigned char color = rainbow_colors[(frame / 10 + arc) % 7];
        
        // Draw arc
        for (int x = 8; x < 32; x++) {
            // Parabolic shape
            int arc_offset = ((x - 20) * (x - 20)) / 40;
            if (y + arc_offset < 12) {
                text40_putchar_color(x, y + arc_offset, '=', color, COLOR_PINK);
            }
        }
    }
}

// Draw unicorn
void draw_unicorn(int x, int y, int facing_right, int frame) {
    // Unicorn body (simplified ASCII art)
    if (facing_right) {
        // Horn (golden)
        text40_putchar_color(x + 1, y - 2, '/', COLOR_YELLOW, COLOR_PINK);
        
        // Head
        text40_putchar_color(x + 2, y - 1, 'o', COLOR_WHITE, COLOR_PINK);
        
        // Neck and body
        text40_puts_color(x, y, "~~O", COLOR_WHITE, COLOR_LIGHT_GRAY);
        text40_puts_color(x - 1, y + 1, "/||\\", COLOR_WHITE, COLOR_LIGHT_GRAY);
        
        // Legs (animated)
        if (frame % 4 < 2) {
            text40_putchar_color(x - 1, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x + 1, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x + 2, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
        } else {
            text40_putchar_color(x - 1, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x + 1, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x + 2, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
        }
        
        // Flowing mane (rainbow colors)
        unsigned char mane_colors[] = {COLOR_PINK, COLOR_PEACH, COLOR_YELLOW};
        text40_putchar_color(x - 2, y - 1, '~', mane_colors[frame % 3], COLOR_PINK);
        text40_putchar_color(x - 2, y, '~', mane_colors[(frame + 1) % 3], COLOR_LIGHT_GRAY);
        
        // Tail
        text40_putchar_color(x - 3, y, '~', COLOR_PINK, COLOR_LIGHT_GRAY);
        text40_putchar_color(x - 3, y + 1, '~', COLOR_PEACH, COLOR_LIGHT_GRAY);
    } else {
        // Horn (golden)
        text40_putchar_color(x - 1, y - 2, '\\', COLOR_YELLOW, COLOR_PINK);
        
        // Head
        text40_putchar_color(x - 2, y - 1, 'o', COLOR_WHITE, COLOR_PINK);
        
        // Neck and body
        text40_puts_color(x - 2, y, "O~~", COLOR_WHITE, COLOR_LIGHT_GRAY);
        text40_puts_color(x - 2, y + 1, "/||\\", COLOR_WHITE, COLOR_LIGHT_GRAY);
        
        // Legs (animated)
        if (frame % 4 < 2) {
            text40_putchar_color(x - 2, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x - 1, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x + 1, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
        } else {
            text40_putchar_color(x - 2, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x - 1, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x, y + 2, '\\', COLOR_WHITE, COLOR_LIGHT_GRAY);
            text40_putchar_color(x + 1, y + 2, '/', COLOR_WHITE, COLOR_LIGHT_GRAY);
        }
        
        // Flowing mane (rainbow colors)
        unsigned char mane_colors[] = {COLOR_PINK, COLOR_PEACH, COLOR_YELLOW};
        text40_putchar_color(x + 2, y - 1, '~', mane_colors[frame % 3], COLOR_PINK);
        text40_putchar_color(x + 2, y, '~', mane_colors[(frame + 1) % 3], COLOR_LIGHT_GRAY);
        
        // Tail
        text40_putchar_color(x + 3, y, '~', COLOR_PINK, COLOR_LIGHT_GRAY);
        text40_putchar_color(x + 3, y + 1, '~', COLOR_PEACH, COLOR_LIGHT_GRAY);
    }
    
    // Magic sparkles around unicorn
    if (frame % 5 == 0) {
        spawn_sparkle(x + rand_range(5) - 2, y + rand_range(4) - 2, COLOR_YELLOW);
    }
}

// Draw hearts
void draw_hearts() {
    for (int i = 0; i < MAX_HEARTS; i++) {
        if (!hearts[i].active) continue;
        
        if (hearts[i].x >= 0 && hearts[i].x < SCREEN_WIDTH &&
            hearts[i].y >= 0 && hearts[i].y < SCREEN_HEIGHT) {
            
            if (hearts[i].size == 0) {
                // Small heart
                text40_putchar_color(hearts[i].x, hearts[i].y, 'v', 
                                   hearts[i].color, COLOR_BLACK);
            } else if (hearts[i].size == 1) {
                // Medium heart
                if (hearts[i].x > 0 && hearts[i].x < SCREEN_WIDTH - 1) {
                    text40_putchar_color(hearts[i].x - 1, hearts[i].y, '<', 
                                       hearts[i].color, COLOR_BLACK);
                    text40_putchar_color(hearts[i].x + 1, hearts[i].y, '>', 
                                       hearts[i].color, COLOR_BLACK);
                }
            } else {
                // Large heart (if space allows)
                if (hearts[i].x > 1 && hearts[i].x < SCREEN_WIDTH - 2 &&
                    hearts[i].y > 0) {
                    text40_puts_color(hearts[i].x - 1, hearts[i].y - 1, "< >", 
                                    hearts[i].color, COLOR_BLACK);
                    text40_putchar_color(hearts[i].x, hearts[i].y, 'V', 
                                       hearts[i].color, COLOR_BLACK);
                }
            }
        }
    }
}

// Draw sparkles
void draw_sparkles() {
    for (int i = 0; i < MAX_SPARKLES; i++) {
        if (!sparkles[i].active || sparkles[i].lifetime <= 0) continue;
        
        if (sparkles[i].x >= 0 && sparkles[i].x < SCREEN_WIDTH &&
            sparkles[i].y >= 0 && sparkles[i].y < SCREEN_HEIGHT) {
            
            char sparkle_char;
            if (sparkles[i].type == 0) {
                sparkle_char = '*';
            } else if (sparkles[i].type == 1) {
                sparkle_char = '+';
            } else {
                sparkle_char = '.';
            }
            
            // Flicker effect
            if (sparkles[i].lifetime > 5 || sparkles[i].lifetime % 2 == 0) {
                text40_putchar_color(sparkles[i].x, sparkles[i].y, sparkle_char, 
                                   sparkles[i].color, COLOR_BLACK);
            }
        }
    }
}

// Draw butterflies
void draw_butterflies() {
    for (int i = 0; i < MAX_BUTTERFLIES; i++) {
        if (!butterflies[i].active) continue;
        
        if (butterflies[i].x >= 1 && butterflies[i].x < SCREEN_WIDTH - 1 &&
            butterflies[i].y >= 0 && butterflies[i].y < SCREEN_HEIGHT) {
            
            // Butterfly colors
            unsigned char butterfly_colors[] = {COLOR_PINK, COLOR_PEACH, COLOR_YELLOW};
            unsigned char color = butterfly_colors[i % 3];
            
            if (butterflies[i].wing_state == 0) {
                // Wings open
                text40_putchar_color(butterflies[i].x - 1, butterflies[i].y, '{', 
                                   color, COLOR_BLACK);
                text40_putchar_color(butterflies[i].x, butterflies[i].y, 'o', 
                                   COLOR_WHITE, COLOR_BLACK);
                text40_putchar_color(butterflies[i].x + 1, butterflies[i].y, '}', 
                                   color, COLOR_BLACK);
            } else {
                // Wings closed
                text40_putchar_color(butterflies[i].x, butterflies[i].y, '|', 
                                   color, COLOR_BLACK);
            }
        }
    }
}

// Draw flowers
void draw_flowers() {
    for (int i = 0; i < MAX_FLOWERS; i++) {
        if (!flowers[i].active) continue;
        
        // Flower bloom animation
        flowers[i].bloom_state = (flowers[i].bloom_state + 1) % 30;
        
        if (flowers[i].bloom_state < 20) {
            // Flower
            text40_putchar_color(flowers[i].x, flowers[i].y, '@', 
                               flowers[i].color, COLOR_LIGHT_GRAY);
            // Stem
            if (flowers[i].y < SCREEN_HEIGHT - 1) {
                text40_putchar_color(flowers[i].x, flowers[i].y + 1, '|', 
                                   COLOR_GREEN, COLOR_LIGHT_GRAY);
            }
            // Leaves
            if (flowers[i].x > 0 && flowers[i].x < SCREEN_WIDTH - 1 &&
                flowers[i].y < SCREEN_HEIGHT - 1) {
                if (i % 2 == 0) {
                    text40_putchar_color(flowers[i].x - 1, flowers[i].y + 1, ',', 
                                       COLOR_DARK_GREEN, COLOR_LIGHT_GRAY);
                } else {
                    text40_putchar_color(flowers[i].x + 1, flowers[i].y + 1, ',', 
                                       COLOR_DARK_GREEN, COLOR_LIGHT_GRAY);
                }
            }
        } else {
            // Closed bud
            text40_putchar_color(flowers[i].x, flowers[i].y, 'o', 
                               flowers[i].color, COLOR_LIGHT_GRAY);
        }
    }
}

// Draw meadow ground
void draw_meadow() {
    // Grass gradient
    for (int y = 18; y < SCREEN_HEIGHT - 2; y++) {
        unsigned char grass_color = (y < 20) ? COLOR_GREEN : COLOR_DARK_GREEN;
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            // Grass texture
            char grass_char = ' ';
            if ((x + y) % 7 == 0) grass_char = ',';
            else if ((x + y) % 11 == 0) grass_char = '.';
            
            text40_putchar_color(x, y, grass_char, COLOR_DARK_GREEN, grass_color);
        }
    }
}

// Draw cupid's arrow
void draw_cupid_arrow(int x, int y) {
    if (x > 2 && x < SCREEN_WIDTH - 1 && y >= 0 && y < SCREEN_HEIGHT) {
        text40_puts_color(x - 2, y, ">>--", COLOR_RED, COLOR_BLACK);
        text40_putchar_color(x + 2, y, '>', COLOR_PINK, COLOR_BLACK);
    }
}

// Draw love messages
void draw_love_messages(int frame) {
    if (frame > 50 && frame < 100) {
        text40_puts_color(12, 10, "Happy Valentine's", COLOR_RED, COLOR_PINK);
        text40_puts_color(16, 11, "Day!", COLOR_WHITE, COLOR_PINK);
    } else if (frame > 150 && frame < 200) {
        text40_puts_color(10, 10, "Love is Magical", COLOR_PINK, COLOR_WHITE);
    } else if (frame > 250 && frame < 300) {
        text40_puts_color(11, 10, "Be My Valentine", COLOR_RED, COLOR_PEACH);
    } else if (frame > 350 && frame < 400) {
        text40_puts_color(9, 10, "Unicorns & Rainbows", COLOR_PINK, COLOR_LIGHT_GRAY);
    }
    
    // Permanent title
    if (frame < 30 || frame > 470) {
        text40_puts_color(8, 1, "VALENTINE'S UNICORN LAND", COLOR_WHITE, COLOR_PINK);
    }
}

// Draw floating cupid
void draw_cupid(int x, int y, int frame) {
    // Simple cupid with bow
    text40_putchar_color(x, y - 1, 'o', COLOR_PEACH, COLOR_BLACK);  // Head
    text40_putchar_color(x - 1, y, '<', COLOR_WHITE, COLOR_BLACK);  // Wing
    text40_putchar_color(x, y, 'O', COLOR_PEACH, COLOR_BLACK);      // Body
    text40_putchar_color(x + 1, y, '>', COLOR_WHITE, COLOR_BLACK);  // Wing
    
    // Bow and arrow
    if (frame % 30 < 15) {
        text40_putchar_color(x + 2, y, ')', COLOR_BROWN, COLOR_BLACK);
        text40_putchar_color(x + 3, y, '-', COLOR_BROWN, COLOR_BLACK);
    }
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
    
    // Initialize scene
    init_hearts();
    init_sparkles();
    init_butterflies();
    init_flowers();
    
    // Main animation loop
    for (int frame = 0; frame < 500; frame++) {
        // Update animations
        update_hearts();
        update_sparkles();
        update_butterflies();
        
        // Update unicorn positions (gentle movement)
        unicorn1_frame = frame;
        unicorn2_frame = frame;
        
        // Unicorns move toward each other
        if (frame > 100 && frame < 250) {
            if (frame % 5 == 0 && unicorn1_x < 18) {
                unicorn1_x++;
            }
            if (frame % 5 == 0 && unicorn2_x > 20) {
                unicorn2_x--;
            }
        }
        
        // Clear screen
        display_clear();
        
        // Draw scene layers (back to front)
        draw_sky();
        draw_rainbow(frame);
        draw_meadow();
        draw_flowers();
        
        // Draw unicorns
        draw_unicorn(unicorn1_x, unicorn1_y, 1, unicorn1_frame);  // Facing right
        draw_unicorn(unicorn2_x, unicorn2_y, 0, unicorn2_frame);  // Facing left
        
        // Draw cupid occasionally
        if (frame > 50 && frame < 150) {
            int cupid_x = 15 + (frame / 10) % 10;
            draw_cupid(cupid_x, 5, frame);
            
            // Shoot arrow
            if (frame == 100) {
                draw_cupid_arrow(cupid_x + 3, 5);
            }
        }
        
        // Draw hearts between unicorns when they're close
        if (unicorn1_x >= 17 && unicorn2_x <= 21) {
            // Big heart between them
            text40_puts_color(18, unicorn1_y - 3, " ** ", COLOR_RED, COLOR_BLACK);
            text40_puts_color(18, unicorn1_y - 2, "****", COLOR_RED, COLOR_BLACK);
            text40_puts_color(18, unicorn1_y - 1, " ** ", COLOR_RED, COLOR_BLACK);
            
            // Extra sparkles
            for (int i = 0; i < 5; i++) {
                spawn_sparkle(15 + rand_range(10), 13 + rand_range(5), COLOR_PINK);
            }
        }
        
        // Draw foreground elements
        draw_butterflies();
        draw_hearts();
        draw_sparkles();
        draw_love_messages(frame);
        
        // Special effects at certain frames
        if (frame == 200) {
            // Heart explosion
            for (int i = 0; i < MAX_HEARTS; i++) {
                if (!hearts[i].active) {
                    hearts[i].x = 20 + rand_range(5) - 2;
                    hearts[i].y = 12;
                    hearts[i].vx = rand_range(5) - 2;
                    hearts[i].vy = -1 - rand_range(2);
                    hearts[i].active = 1;
                }
            }
        }
        
        // Magical sparkle trail from unicorn horns
        if (frame % 3 == 0) {
            spawn_sparkle(unicorn1_x + 1, unicorn1_y - 2, COLOR_YELLOW);
            spawn_sparkle(unicorn2_x - 1, unicorn2_y - 2, COLOR_YELLOW);
        }
        
        // Bottom status bar
        text40_puts_color(1, SCREEN_HEIGHT - 1, "Love", COLOR_RED, COLOR_BLACK);
        text40_puts_color(6, SCREEN_HEIGHT - 1, "Magic", COLOR_PINK, COLOR_BLACK);
        text40_puts_color(12, SCREEN_HEIGHT - 1, "Unicorns", COLOR_WHITE, COLOR_BLACK);
        text40_puts_color(21, SCREEN_HEIGHT - 1, "Forever", COLOR_PEACH, COLOR_BLACK);
        text40_puts_color(29, SCREEN_HEIGHT - 1, "Valentine", COLOR_RED, COLOR_BLACK);
        
        display_flush();
        delay(3000); // Animation speed
        
        // Add new hearts occasionally
        if (frame % 20 == 0) {
            for (int i = 0; i < MAX_HEARTS; i++) {
                if (!hearts[i].active) {
                    hearts[i].x = rand_range(SCREEN_WIDTH);
                    hearts[i].y = SCREEN_HEIGHT - 2;
                    hearts[i].vx = rand_range(3) - 1;
                    hearts[i].vy = -1;
                    hearts[i].active = 1;
                    break;
                }
            }
        }
    }
    
    // End sequence - final romantic scene
    display_clear();
    
    // Draw final message with hearts
    text40_puts_color(10, 8, "Happy Valentine's Day!", COLOR_RED, COLOR_PINK);
    text40_puts_color(13, 10, "Love & Magic", COLOR_WHITE, COLOR_PEACH);
    text40_puts_color(11, 12, "Unicorns Forever", COLOR_PINK, COLOR_WHITE);
    
    // Big heart
    text40_puts_color(16, 15, "  ***  ", COLOR_RED, COLOR_BLACK);
    text40_puts_color(16, 16, " ***** ", COLOR_RED, COLOR_BLACK);
    text40_puts_color(16, 17, "*******", COLOR_RED, COLOR_BLACK);
    text40_puts_color(16, 18, " ***** ", COLOR_RED, COLOR_BLACK);
    text40_puts_color(16, 19, "  ***  ", COLOR_RED, COLOR_BLACK);
    text40_puts_color(16, 20, "   *   ", COLOR_RED, COLOR_BLACK);
    
    // Sparkles around
    for (int i = 0; i < 20; i++) {
        int x = rand_range(SCREEN_WIDTH);
        int y = rand_range(SCREEN_HEIGHT);
        char sparkle = (rand_range(3) == 0) ? '*' : '.';
        text40_putchar_color(x, y, sparkle, COLOR_YELLOW, COLOR_BLACK);
    }
    
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Valentine's unicorn magic complete!");
    
    return 0;
}