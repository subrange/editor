// Gothic Vampire Castle - Dark Romance Animation
// Features castle silhouette, bats, fog, moon phases, and gothic atmosphere

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25
#define MAX_BATS 15
#define MAX_FOG 20
#define MAX_PARTICLES 25
#define MAX_CANDLES 8
#define MAX_ROSES 5

// Bat arrays (no structs)
int bat_x[MAX_BATS];
int bat_y[MAX_BATS];
int bat_vx[MAX_BATS];
int bat_vy[MAX_BATS];
int bat_wing_state[MAX_BATS];
int bat_active[MAX_BATS];

// Fog arrays
int fog_x[MAX_FOG];
int fog_y[MAX_FOG];
int fog_density[MAX_FOG];
int fog_active[MAX_FOG];

// Particle arrays (for rose petals, candle sparks)
int particle_x[MAX_PARTICLES];
int particle_y[MAX_PARTICLES];
int particle_vx[MAX_PARTICLES];
int particle_vy[MAX_PARTICLES];
int particle_life[MAX_PARTICLES];
int particle_type[MAX_PARTICLES]; // 0=petal, 1=spark, 2=blood drop
unsigned char particle_color[MAX_PARTICLES];

// Candle arrays
int candle_x[MAX_CANDLES];
int candle_y[MAX_CANDLES];
int candle_flicker[MAX_CANDLES];
int candle_active[MAX_CANDLES];

// Rose arrays
int rose_x[MAX_ROSES];
int rose_y[MAX_ROSES];
int rose_petal_timer[MAX_ROSES];
int rose_active[MAX_ROSES];

// Global animation state
int moon_phase = 0;
int lightning_timer = 0;
int wind_offset = 0;
int frame_count = 0;

// Get random number in range
int rand_range(int max) {
    return (rng_get() % max);
}

// Initialize bats
void init_bats() {
    for (int i = 0; i < MAX_BATS; i++) {
        if (i < 8) { // Start with 8 bats
            bat_x[i] = rand_range(SCREEN_WIDTH);
            bat_y[i] = 2 + rand_range(10);
            bat_vx[i] = (rand_range(2) == 0) ? 1 : -1;
            bat_vy[i] = 0;
            bat_wing_state[i] = rand_range(2);
            bat_active[i] = 1;
        } else {
            bat_active[i] = 0;
        }
    }
}

// Initialize fog
void init_fog() {
    for (int i = 0; i < MAX_FOG; i++) {
        fog_x[i] = rand_range(SCREEN_WIDTH);
        fog_y[i] = SCREEN_HEIGHT - 5 + rand_range(5);
        fog_density[i] = rand_range(3);
        fog_active[i] = 1;
    }
}

// Initialize candles
void init_candles() {
    // Place candles around the scene
    candle_x[0] = 5; candle_y[0] = 20;
    candle_x[1] = 35; candle_y[1] = 20;
    candle_x[2] = 10; candle_y[2] = 18;
    candle_x[3] = 30; candle_y[3] = 18;
    candle_x[4] = 15; candle_y[4] = 21;
    candle_x[5] = 25; candle_y[5] = 21;
    candle_x[6] = 20; candle_y[6] = 19;
    candle_x[7] = 20; candle_y[7] = 22;
    
    for (int i = 0; i < MAX_CANDLES; i++) {
        candle_flicker[i] = rand_range(3);
        candle_active[i] = 1;
    }
}

// Initialize roses
void init_roses() {
    rose_x[0] = 8; rose_y[0] = 19;
    rose_x[1] = 32; rose_y[1] = 19;
    rose_x[2] = 20; rose_y[2] = 20;
    rose_x[3] = 12; rose_y[3] = 21;
    rose_x[4] = 28; rose_y[4] = 21;
    
    for (int i = 0; i < MAX_ROSES; i++) {
        rose_petal_timer[i] = rand_range(100);
        rose_active[i] = 1;
    }
}

// Update bats
void update_bats() {
    for (int i = 0; i < MAX_BATS; i++) {
        if (!bat_active[i]) continue;
        
        // Flap wings
        bat_wing_state[i] = (bat_wing_state[i] + 1) % 2;
        
        // Move in figure-8 pattern
        bat_x[i] += bat_vx[i];
        
        // Sine wave vertical movement
        if (frame_count % 3 == 0) {
            bat_vy[i] = (rand_range(3) - 1);
            bat_y[i] += bat_vy[i];
        }
        
        // Bounce off edges
        if (bat_x[i] <= 0 || bat_x[i] >= SCREEN_WIDTH - 1) {
            bat_vx[i] = -bat_vx[i];
        }
        
        // Keep in upper area
        if (bat_y[i] < 1) bat_y[i] = 1;
        if (bat_y[i] > 12) bat_y[i] = 12;
    }
}

// Update fog
void update_fog() {
    for (int i = 0; i < MAX_FOG; i++) {
        if (!fog_active[i]) continue;
        
        // Drift with wind
        fog_x[i] += ((frame_count / 10) % 3 == 0) ? 1 : 0;
        
        // Wrap around screen
        if (fog_x[i] >= SCREEN_WIDTH) {
            fog_x[i] = 0;
            fog_y[i] = SCREEN_HEIGHT - 5 + rand_range(5);
        }
    }
}

// Update particles
void update_particles() {
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] <= 0) continue;
        
        particle_x[i] += particle_vx[i];
        particle_y[i] += particle_vy[i];
        particle_life[i]--;
        
        // Gravity for petals
        if (particle_type[i] == 0) {
            if (frame_count % 2 == 0) {
                particle_vy[i] = 1;
            }
        }
    }
}

// Spawn rose petal
void spawn_petal(int x, int y) {
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] <= 0) {
            particle_x[i] = x;
            particle_y[i] = y;
            particle_vx[i] = rand_range(3) - 1;
            particle_vy[i] = 0;
            particle_life[i] = 30 + rand_range(20);
            particle_type[i] = 0;
            particle_color[i] = (rand_range(2) == 0) ? COLOR_RED : COLOR_DARK_PURPLE;
            break;
        }
    }
}

// Spawn candle spark
void spawn_spark(int x, int y) {
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] <= 0) {
            particle_x[i] = x;
            particle_y[i] = y;
            particle_vx[i] = rand_range(3) - 1;
            particle_vy[i] = -1;
            particle_life[i] = 10 + rand_range(10);
            particle_type[i] = 1;
            particle_color[i] = (rand_range(3) == 0) ? COLOR_YELLOW : COLOR_ORANGE;
            break;
        }
    }
}

// Draw night sky
void draw_sky() {
    // Gradient sky
    for (int y = 0; y < 13; y++) {
        unsigned char sky_color = COLOR_BLACK;
        if (y < 3) {
            sky_color = COLOR_BLACK;
        } else if (y < 6) {
            sky_color = COLOR_DARK_PURPLE;
        } else if (y < 9) {
            sky_color = COLOR_DARK_BLUE;
        } else {
            sky_color = COLOR_BLACK;
        }
        
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            text40_putchar_color(x, y, ' ', COLOR_BLACK, sky_color);
        }
    }
    
    // Stars
    for (int i = 0; i < 15; i++) {
        int x = (i * 7 + frame_count / 20) % SCREEN_WIDTH;
        int y = (i * 3) % 8;
        if (frame_count % 30 < 25 || i % 3 == 0) {
            text40_putchar_color(x, y, '.', COLOR_WHITE, COLOR_BLACK);
        }
    }
}

// Draw moon
void draw_moon() {
    int moon_x = 32;
    int moon_y = 3;
    
    // Moon phases
    moon_phase = (frame_count / 50) % 4;
    
    if (moon_phase == 0) { // Full moon
        text40_puts_color(moon_x, moon_y - 1, " __ ", COLOR_WHITE, COLOR_BLACK);
        text40_puts_color(moon_x, moon_y, "(  )", COLOR_WHITE, COLOR_BLACK);
        text40_puts_color(moon_x, moon_y + 1, " -- ", COLOR_LIGHT_GRAY, COLOR_BLACK);
    } else if (moon_phase == 1) { // Waning
        text40_puts_color(moon_x, moon_y - 1, " _ ", COLOR_LIGHT_GRAY, COLOR_BLACK);
        text40_puts_color(moon_x, moon_y, "( )", COLOR_LIGHT_GRAY, COLOR_BLACK);
        text40_puts_color(moon_x, moon_y + 1, " - ", COLOR_DARK_GRAY, COLOR_BLACK);
    } else if (moon_phase == 2) { // Half moon
        text40_puts_color(moon_x, moon_y - 1, " _", COLOR_LIGHT_GRAY, COLOR_BLACK);
        text40_puts_color(moon_x, moon_y, "( ", COLOR_LIGHT_GRAY, COLOR_BLACK);
        text40_puts_color(moon_x, moon_y + 1, " -", COLOR_DARK_GRAY, COLOR_BLACK);
    } else { // Crescent
        text40_puts_color(moon_x, moon_y, "(", COLOR_DARK_GRAY, COLOR_BLACK);
    }
    
    // Moonlight rays
    if (moon_phase == 0 && frame_count % 20 < 10) {
        text40_putchar_color(moon_x - 2, moon_y + 2, '\\', COLOR_DARK_GRAY, COLOR_BLACK);
        text40_putchar_color(moon_x + 4, moon_y + 2, '/', COLOR_DARK_GRAY, COLOR_BLACK);
    }
}

// Draw gothic castle
void draw_castle() {
    // Castle towers and walls
    int castle_base = 13;
    
    // Left tower
    text40_puts_color(3, castle_base - 5, "  ||", COLOR_DARK_GRAY, COLOR_BLACK);
    // Draw tower roof character by character to avoid backslash issues
    text40_putchar_color(3, castle_base - 4, ' ', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(4, castle_base - 4, '/', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(5, castle_base - 4, '|', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(6, castle_base - 4, '|', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(7, castle_base - 4, '\\', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(3, castle_base - 3, " ||||", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(3, castle_base - 2, "[||||]", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(3, castle_base - 1, "[|**|]", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(3, castle_base, "[|**|]", COLOR_DARK_GRAY, COLOR_BLACK);
    
    // Center castle
    text40_puts_color(14, castle_base - 7, "    A", COLOR_DARK_GRAY, COLOR_BLACK);
    // Draw roofs character by character
    text40_puts_color(14, castle_base - 6, "   /", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(18, castle_base - 6, '|', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(19, castle_base - 6, '\\', COLOR_DARK_GRAY, COLOR_BLACK);
    
    text40_puts_color(14, castle_base - 5, "  /", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(17, castle_base - 5, "|||", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(20, castle_base - 5, '\\', COLOR_DARK_GRAY, COLOR_BLACK);
    
    text40_puts_color(14, castle_base - 4, " /", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(16, castle_base - 4, "|||||", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(21, castle_base - 4, '\\', COLOR_DARK_GRAY, COLOR_BLACK);
    
    text40_puts_color(14, castle_base - 3, "[|||||||]", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(14, castle_base - 2, "[||***||]", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(14, castle_base - 1, "[||***||]", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(14, castle_base, "[|||D|||]", COLOR_DARK_GRAY, COLOR_BLACK);
    
    // Right tower
    text40_puts_color(30, castle_base - 5, "||", COLOR_DARK_GRAY, COLOR_BLACK);
    // Draw roof character by character
    text40_putchar_color(29, castle_base - 4, '/', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(30, castle_base - 4, '|', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(31, castle_base - 4, '|', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(32, castle_base - 4, '\\', COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(29, castle_base - 3, "||||", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(28, castle_base - 2, "[||||]", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(28, castle_base - 1, "[|**|]", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_puts_color(28, castle_base, "[|**|]", COLOR_DARK_GRAY, COLOR_BLACK);
    
    // Connecting walls
    for (int x = 9; x < 14; x++) {
        text40_putchar_color(x, castle_base - 1, '=', COLOR_DARK_GRAY, COLOR_BLACK);
        text40_putchar_color(x, castle_base, '=', COLOR_DARK_GRAY, COLOR_BLACK);
    }
    for (int x = 23; x < 28; x++) {
        text40_putchar_color(x, castle_base - 1, '=', COLOR_DARK_GRAY, COLOR_BLACK);
        text40_putchar_color(x, castle_base, '=', COLOR_DARK_GRAY, COLOR_BLACK);
    }
    
    // Windows with candlelight
    if (frame_count % 40 < 30) {
        text40_putchar_color(5, castle_base - 3, '*', COLOR_YELLOW, COLOR_DARK_GRAY);
        text40_putchar_color(31, castle_base - 3, '*', COLOR_YELLOW, COLOR_DARK_GRAY);
    }
    if (frame_count % 50 < 35) {
        text40_putchar_color(17, castle_base - 5, '*', COLOR_ORANGE, COLOR_DARK_GRAY);
        text40_putchar_color(19, castle_base - 5, '*', COLOR_ORANGE, COLOR_DARK_GRAY);
    }
}

// Draw bats
void draw_bats() {
    for (int i = 0; i < MAX_BATS; i++) {
        if (!bat_active[i]) continue;
        
        if (bat_x[i] >= 0 && bat_x[i] < SCREEN_WIDTH &&
            bat_y[i] >= 0 && bat_y[i] < SCREEN_HEIGHT) {
            
            if (bat_wing_state[i] == 0) {
                // Wings up: ^v^
                if (bat_x[i] > 0) 
                    text40_putchar_color(bat_x[i] - 1, bat_y[i], '^', COLOR_BLACK, COLOR_BLACK);
                text40_putchar_color(bat_x[i], bat_y[i], 'v', COLOR_BLACK, COLOR_BLACK);
                if (bat_x[i] < SCREEN_WIDTH - 1)
                    text40_putchar_color(bat_x[i] + 1, bat_y[i], '^', COLOR_BLACK, COLOR_BLACK);
            } else {
                // Wings down: -v-
                if (bat_x[i] > 0)
                    text40_putchar_color(bat_x[i] - 1, bat_y[i], '-', COLOR_BLACK, COLOR_BLACK);
                text40_putchar_color(bat_x[i], bat_y[i], 'v', COLOR_BLACK, COLOR_BLACK);
                if (bat_x[i] < SCREEN_WIDTH - 1)
                    text40_putchar_color(bat_x[i] + 1, bat_y[i], '-', COLOR_BLACK, COLOR_BLACK);
            }
        }
    }
}

// Draw fog
void draw_fog() {
    for (int i = 0; i < MAX_FOG; i++) {
        if (!fog_active[i]) continue;
        
        if (fog_x[i] >= 0 && fog_x[i] < SCREEN_WIDTH &&
            fog_y[i] >= 0 && fog_y[i] < SCREEN_HEIGHT) {
            
            char fog_char = '~';
            unsigned char fog_color = COLOR_LIGHT_GRAY;
            
            if (fog_density[i] == 0) {
                fog_char = '.';
                fog_color = COLOR_DARK_GRAY;
            } else if (fog_density[i] == 1) {
                fog_char = '~';
                fog_color = COLOR_LIGHT_GRAY;
            } else {
                fog_char = '=';
                fog_color = COLOR_WHITE;
            }
            
            text40_putchar_color(fog_x[i], fog_y[i], fog_char, fog_color, COLOR_BLACK);
        }
    }
}

// Draw candles
void draw_candles() {
    for (int i = 0; i < MAX_CANDLES; i++) {
        if (!candle_active[i]) continue;
        
        // Candle body
        text40_putchar_color(candle_x[i], candle_y[i], '|', COLOR_WHITE, COLOR_BLACK);
        
        // Flame (flickering)
        candle_flicker[i] = (candle_flicker[i] + 1) % 3;
        unsigned char flame_color = COLOR_YELLOW;
        char flame_char = '*';
        
        if (candle_flicker[i] == 0) {
            flame_color = COLOR_ORANGE;
            flame_char = '*';
        } else if (candle_flicker[i] == 1) {
            flame_color = COLOR_YELLOW;
            flame_char = 'o';
        } else {
            flame_color = COLOR_RED;
            flame_char = '.';
        }
        
        text40_putchar_color(candle_x[i], candle_y[i] - 1, flame_char, flame_color, COLOR_BLACK);
        
        // Occasional spark
        if (rand_range(100) < 5) {
            spawn_spark(candle_x[i], candle_y[i] - 1);
        }
    }
}

// Draw roses
void draw_roses() {
    for (int i = 0; i < MAX_ROSES; i++) {
        if (!rose_active[i]) continue;
        
        // Rose bloom
        text40_putchar_color(rose_x[i], rose_y[i], '@', COLOR_RED, COLOR_BLACK);
        
        // Stem
        if (rose_y[i] < SCREEN_HEIGHT - 1) {
            text40_putchar_color(rose_x[i], rose_y[i] + 1, '|', COLOR_DARK_GREEN, COLOR_BLACK);
        }
        
        // Falling petals occasionally
        rose_petal_timer[i]--;
        if (rose_petal_timer[i] <= 0) {
            spawn_petal(rose_x[i], rose_y[i]);
            rose_petal_timer[i] = 80 + rand_range(40);
        }
    }
}

// Draw particles
void draw_particles() {
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] <= 0) continue;
        
        if (particle_x[i] >= 0 && particle_x[i] < SCREEN_WIDTH &&
            particle_y[i] >= 0 && particle_y[i] < SCREEN_HEIGHT) {
            
            char p_char = '.';
            if (particle_type[i] == 0) { // Petal
                p_char = (particle_life[i] > 10) ? 'o' : '.';
            } else if (particle_type[i] == 1) { // Spark
                p_char = (particle_life[i] > 5) ? '*' : '.';
            }
            
            text40_putchar_color(particle_x[i], particle_y[i], p_char, 
                               particle_color[i], COLOR_BLACK);
        }
    }
}

// Draw vampire silhouette
void draw_vampire(int x, int y) {
    // Cape flowing
    int cape_sway = (frame_count / 5) % 3;
    
    // Head
    text40_putchar_color(x, y - 2, 'O', COLOR_WHITE, COLOR_BLACK);
    
    // Body and cape
    text40_putchar_color(x, y - 1, '|', COLOR_BLACK, COLOR_BLACK);
    
    // Cape edges
    if (cape_sway == 0) {
        text40_putchar_color(x - 1, y - 1, '\\', COLOR_BLACK, COLOR_BLACK);
        text40_putchar_color(x + 1, y - 1, '/', COLOR_BLACK, COLOR_BLACK);
        text40_putchar_color(x - 2, y, '\\', COLOR_DARK_PURPLE, COLOR_BLACK);
        text40_putchar_color(x + 2, y, '/', COLOR_DARK_PURPLE, COLOR_BLACK);
    } else if (cape_sway == 1) {
        text40_putchar_color(x - 1, y - 1, '\\', COLOR_BLACK, COLOR_BLACK);
        text40_putchar_color(x + 1, y - 1, '/', COLOR_BLACK, COLOR_BLACK);
        text40_putchar_color(x - 2, y, '(', COLOR_DARK_PURPLE, COLOR_BLACK);
        text40_putchar_color(x + 2, y, ')', COLOR_DARK_PURPLE, COLOR_BLACK);
    } else {
        text40_putchar_color(x - 1, y - 1, '(', COLOR_BLACK, COLOR_BLACK);
        text40_putchar_color(x + 1, y - 1, ')', COLOR_BLACK, COLOR_BLACK);
        text40_putchar_color(x - 2, y, '\\', COLOR_DARK_PURPLE, COLOR_BLACK);
        text40_putchar_color(x + 2, y, '/', COLOR_DARK_PURPLE, COLOR_BLACK);
    }
    
    // Base of cape
    text40_putchar_color(x - 1, y, 'V', COLOR_BLACK, COLOR_BLACK);
    text40_putchar_color(x, y, 'V', COLOR_BLACK, COLOR_BLACK);
    text40_putchar_color(x + 1, y, 'V', COLOR_BLACK, COLOR_BLACK);
}

// Draw romantic text overlay
void draw_romantic_text() {
    if (frame_count > 100 && frame_count < 150) {
        text40_puts_color(10, 16, "Eternal Night...", COLOR_RED, COLOR_BLACK);
    } else if (frame_count > 200 && frame_count < 250) {
        text40_puts_color(8, 16, "Forever in Darkness", COLOR_DARK_PURPLE, COLOR_BLACK);
    } else if (frame_count > 300 && frame_count < 350) {
        text40_puts_color(11, 16, "Blood & Roses", COLOR_RED, COLOR_BLACK);
    } else if (frame_count > 400 && frame_count < 450) {
        text40_puts_color(9, 16, "Gothic Romance", COLOR_PINK, COLOR_BLACK);
    }
}

// Draw lightning effect
void draw_lightning() {
    if (lightning_timer > 0) {
        lightning_timer--;
        
        // Flash the sky
        if (lightning_timer > 3) {
            for (int y = 0; y < 8; y++) {
                for (int x = 0; x < SCREEN_WIDTH; x++) {
                    text40_putchar_color(x, y, ' ', COLOR_WHITE, COLOR_WHITE);
                }
            }
        } else {
            // Lightning bolt
            int bolt_x = 15 + rand_range(10);
            for (int y = 0; y < 8; y++) {
                text40_putchar_color(bolt_x + (y % 2), y, '/', COLOR_WHITE, COLOR_BLACK);
            }
        }
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
    init_bats();
    init_fog();
    init_candles();
    init_roses();
    
    // Main animation loop
    for (frame_count = 0; frame_count < 500; frame_count++) {
        // Update animations
        update_bats();
        update_fog();
        update_particles();
        
        // Occasional lightning
        if (rand_range(100) < 2) {
            lightning_timer = 5;
        }
        
        // Clear screen
        display_clear();
        
        // Draw scene layers (back to front)
        draw_sky();
        draw_moon();
        draw_lightning();
        draw_castle();
        draw_bats();
        
        // Draw vampire at different positions
        int vamp_x = 20 + ((frame_count / 30) % 3) - 1;
        int vamp_y = 18;
        if (frame_count > 50) {
            draw_vampire(vamp_x, vamp_y);
        }
        
        // Foreground elements
        draw_candles();
        draw_roses();
        draw_particles();
        draw_fog();
        draw_romantic_text();
        
        // Title
        if (frame_count < 50 || frame_count > 450) {
            text40_puts_color(12, 2, "VAMPIRE CASTLE", COLOR_RED, COLOR_BLACK);
            text40_puts_color(10, 3, "A Gothic Romance", COLOR_DARK_PURPLE, COLOR_BLACK);
        }
        
        display_flush();
        delay(3000); // Animation speed
    }
    
    // End sequence
    display_clear();
    
    // Final romantic scene
    text40_puts_color(14, 10, "Forever...", COLOR_RED, COLOR_BLACK);
    text40_puts_color(12, 12, "In the shadows", COLOR_DARK_PURPLE, COLOR_BLACK);
    text40_puts_color(11, 14, "Love never dies", COLOR_PINK, COLOR_BLACK);
    
    // Draw a heart
    text40_puts_color(17, 17, " ** ", COLOR_RED, COLOR_BLACK);
    text40_puts_color(17, 18, "****", COLOR_RED, COLOR_BLACK);
    text40_puts_color(17, 19, " ** ", COLOR_RED, COLOR_BLACK);
    text40_puts_color(18, 20, "*", COLOR_RED, COLOR_BLACK);
    
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Gothic romance complete!");
    
    return 0;
}