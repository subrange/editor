// Noir cyberpunk car driving on autobahn with night city
// Features perspective road, rain, neon lights, and city skyline

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

#define MAX_RAINDROPS 30
#define MAX_BUILDINGS 10
#define MAX_ROAD_LINES 8
#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25

// Raindrop structure
struct Raindrop {
    int x;
    int y;
    int speed;
    char active;
};

// Building structure for skyline
struct Building {
    int x;
    int width;
    int height;
    unsigned char color;
};

// Road line structure for perspective
struct RoadLine {
    int y;
    int offset;
};

struct Raindrop rain[MAX_RAINDROPS];
struct Building buildings[MAX_BUILDINGS];
struct RoadLine road_lines[MAX_ROAD_LINES];
int car_x = 20;
int car_sway = 0;

// Get random number in range [0, max)
int rand_range(int max) {
    return (rng_get() % max);
}

// Initialize rain
void init_rain() {
    for (int i = 0; i < MAX_RAINDROPS; i++) {
        rain[i].x = rand_range(SCREEN_WIDTH);
        rain[i].y = rand_range(10);
        rain[i].speed = 1 + rand_range(2);
        rain[i].active = (i < 20) ? 1 : 0;
    }
}

// Initialize city skyline
void init_skyline() {
    int x_pos = 0;
    for (int i = 0; i < MAX_BUILDINGS; i++) {
        buildings[i].x = x_pos;
        buildings[i].width = 3 + rand_range(5);
        buildings[i].height = 4 + rand_range(8);
        
        // Cyberpunk neon colors
        unsigned char neon_colors[] = {
            COLOR_DARK_PURPLE, COLOR_DARK_BLUE, COLOR_INDIGO,
            COLOR_PINK, COLOR_DARK_GREEN
        };
        buildings[i].color = neon_colors[rand_range(5)];
        
        x_pos += buildings[i].width + 1;
        if (x_pos >= SCREEN_WIDTH) break;
    }
}

// Initialize road lines for perspective
void init_road() {
    for (int i = 0; i < MAX_ROAD_LINES; i++) {
        road_lines[i].y = 14 + i;
        road_lines[i].offset = 0;
    }
}

// Update rain animation
void update_rain() {
    for (int i = 0; i < MAX_RAINDROPS; i++) {
        if (!rain[i].active) continue;
        
        rain[i].y += rain[i].speed;
        
        // Reset raindrop when it goes off screen
        if (rain[i].y >= SCREEN_HEIGHT) {
            rain[i].y = 0;
            rain[i].x = rand_range(SCREEN_WIDTH);
            rain[i].speed = 1 + rand_range(2);
        }
    }
}

// Update road animation
void update_road(int frame) {
    for (int i = 0; i < MAX_ROAD_LINES; i++) {
        road_lines[i].offset = (frame * 2) % 8;
    }
}

// Draw city skyline
void draw_skyline() {
    // Sky gradient
    for (int y = 0; y < 12; y++) {
        unsigned char sky_color = (y < 6) ? COLOR_BLACK : COLOR_DARK_BLUE;
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            text40_putchar_color(x, y, ' ', COLOR_BLACK, sky_color);
        }
    }
    
    // Buildings
    for (int i = 0; i < MAX_BUILDINGS; i++) {
        for (int x = 0; x < buildings[i].width; x++) {
            for (int y = 0; y < buildings[i].height; y++) {
                int screen_x = buildings[i].x + x;
                int screen_y = 12 - buildings[i].height + y;
                
                if (screen_x >= 0 && screen_x < SCREEN_WIDTH && 
                    screen_y >= 0 && screen_y < 12) {
                    
                    // Building body
                    char building_char = '#';
                    
                    // Windows (lit up)
                    if ((x % 2 == 1) && (y % 2 == 0) && y < buildings[i].height - 1) {
                        building_char = '.';
                        text40_putchar_color(screen_x, screen_y, building_char,
                                           COLOR_YELLOW, buildings[i].color);
                    } else {
                        text40_putchar_color(screen_x, screen_y, building_char,
                                           COLOR_DARK_GRAY, buildings[i].color);
                    }
                }
            }
        }
    }
    
    // Neon signs on some buildings
    text40_puts_color(5, 8, "NEON", COLOR_PINK, COLOR_BLACK);
    text40_puts_color(25, 6, "2084", COLOR_BLUE, COLOR_BLACK);
    text40_puts_color(15, 9, "CYBER", COLOR_GREEN, COLOR_BLACK);
}

// Draw perspective road
void draw_road(int frame) {
    int road_y_start = 14;
    
    // Road surface with perspective
    for (int y = road_y_start; y < SCREEN_HEIGHT - 2; y++) {
        int perspective = (y - road_y_start);
        int road_left = 10 - perspective;
        int road_right = 30 + perspective;
        
        // Ensure bounds
        if (road_left < 0) road_left = 0;
        if (road_right >= SCREEN_WIDTH) road_right = SCREEN_WIDTH - 1;
        
        // Draw road
        for (int x = road_left; x <= road_right; x++) {
            text40_putchar_color(x, y, ' ', COLOR_BLACK, COLOR_DARK_GRAY);
        }
        
        // Road lines (dashed center line)
        int center = SCREEN_WIDTH / 2;
        if ((y + frame / 2) % 4 < 2) {
            text40_putchar_color(center, y, '|', COLOR_YELLOW, COLOR_DARK_GRAY);
        }
        
        // Edge lines
        if (road_left > 0) {
            text40_putchar_color(road_left, y, '|', COLOR_WHITE, COLOR_DARK_GRAY);
        }
        if (road_right < SCREEN_WIDTH - 1) {
            text40_putchar_color(road_right, y, '|', COLOR_WHITE, COLOR_DARK_GRAY);
        }
    }
    
    // Horizon line
    for (int x = 5; x < 35; x++) {
        text40_putchar_color(x, 13, '=', COLOR_DARK_PURPLE, COLOR_BLACK);
    }
}

// Draw the cyberpunk car
void draw_car(int frame) {
    int y = 19;
    
    // Car body (sleek sports car design)
    //     ___
    //   _/___\_
    //  [|o|||o|]
    //   \-----/
    
    // Top
    text40_puts_color(car_x - 2, y - 1, " ___ ", COLOR_DARK_PURPLE, COLOR_BLACK);
    
    // Middle with windshield
    text40_putchar_color(car_x - 3, y, '_', COLOR_DARK_PURPLE, COLOR_BLACK);
    text40_putchar_color(car_x - 2, y, '/', COLOR_PINK, COLOR_BLACK);
    text40_puts_color(car_x - 1, y, "___", COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(car_x + 2, y, '\\', COLOR_PINK, COLOR_BLACK);
    text40_putchar_color(car_x + 3, y, '_', COLOR_DARK_PURPLE, COLOR_BLACK);
    
    // Body with lights
    text40_putchar_color(car_x - 4, y + 1, '[', COLOR_DARK_PURPLE, COLOR_BLACK);
    text40_putchar_color(car_x - 3, y + 1, '|', COLOR_DARK_PURPLE, COLOR_BLACK);
    text40_putchar_color(car_x - 2, y + 1, 'o', COLOR_RED, COLOR_BLACK);  // Tail light
    text40_puts_color(car_x - 1, y + 1, "|||", COLOR_INDIGO, COLOR_BLACK);
    text40_putchar_color(car_x + 2, y + 1, 'o', COLOR_RED, COLOR_BLACK);  // Tail light
    text40_putchar_color(car_x + 3, y + 1, '|', COLOR_DARK_PURPLE, COLOR_BLACK);
    text40_putchar_color(car_x + 4, y + 1, ']', COLOR_DARK_PURPLE, COLOR_BLACK);
    
    // Bottom with exhaust
    text40_putchar_color(car_x - 3, y + 2, '\\', COLOR_DARK_PURPLE, COLOR_BLACK);
    text40_puts_color(car_x - 2, y + 2, "-----", COLOR_DARK_GRAY, COLOR_BLACK);
    text40_putchar_color(car_x + 3, y + 2, '/', COLOR_DARK_PURPLE, COLOR_BLACK);
    
    // Exhaust flames (animated)
    if (frame % 4 < 2) {
        text40_putchar_color(car_x - 4, y + 2, '~', COLOR_ORANGE, COLOR_BLACK);
        text40_putchar_color(car_x + 4, y + 2, '~', COLOR_ORANGE, COLOR_BLACK);
    }
    
    // Neon underglow
    for (int x = car_x - 3; x <= car_x + 3; x++) {
        text40_putchar_color(x, y + 3, '_', COLOR_PINK, COLOR_BLACK);
    }
}

// Draw rain effect
void draw_rain() {
    for (int i = 0; i < MAX_RAINDROPS; i++) {
        if (!rain[i].active) continue;
        
        if (rain[i].x >= 0 && rain[i].x < SCREEN_WIDTH &&
            rain[i].y >= 0 && rain[i].y < SCREEN_HEIGHT) {
            
            // Don't draw rain on the car
            if (rain[i].y < 18 || rain[i].y > 23 || 
                rain[i].x < car_x - 4 || rain[i].x > car_x + 4) {
                text40_putchar_color(rain[i].x, rain[i].y, '|', 
                                   COLOR_LIGHT_GRAY, COLOR_BLACK);
            }
        }
    }
}

// Draw HUD
void draw_hud() {
    // Speed indicator
    text40_puts_color(1, 23, "SPEED:", COLOR_GREEN, COLOR_BLACK);
    text40_puts_color(8, 23, "280", COLOR_YELLOW, COLOR_BLACK);
    text40_puts_color(12, 23, "KM/H", COLOR_GREEN, COLOR_BLACK);
    
    // Location
    text40_puts_color(25, 23, "AUTOBAHN A9", COLOR_BLUE, COLOR_BLACK);
    
    // Time
    text40_puts_color(1, 24, "02:47 AM", COLOR_PINK, COLOR_BLACK);
    
    // Destination
    text40_puts_color(25, 24, "BERLIN->NEO", COLOR_INDIGO, COLOR_BLACK);
    
    // Radio station
    text40_puts_color(12, 24, "[CYBER.FM]", COLOR_DARK_PURPLE, COLOR_BLACK);
}

// Draw street lights streaking by
void draw_street_lights(int frame) {
    // Left side lights
    for (int i = 0; i < 3; i++) {
        int y = 14 + (frame + i * 8) % 10;
        if (y < 22) {
            text40_putchar_color(5 - (y - 14) / 2, y, '*', COLOR_YELLOW, COLOR_BLACK);
        }
    }
    
    // Right side lights
    for (int i = 0; i < 3; i++) {
        int y = 14 + (frame + i * 8 + 4) % 10;
        if (y < 22) {
            text40_putchar_color(35 + (y - 14) / 2, y, '*', COLOR_YELLOW, COLOR_BLACK);
        }
    }
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Update car position (subtle sway)
void update_car(int frame) {
    // Subtle left-right movement
    car_sway = (frame / 10) % 4;
    if (car_sway == 1) car_x = 19;
    else if (car_sway == 2) car_x = 20;
    else if (car_sway == 3) car_x = 21;
    else car_x = 20;
}

int main() {
    // Enable TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Initialize scene
    init_rain();
    init_skyline();
    init_road();
    
    // Main animation loop
    for (int frame = 0; frame < 300; frame++) {
        // Update animations
        update_rain();
        update_road(frame);
        update_car(frame);
        
        // Clear screen with dark background
        display_clear();
        
        // Draw scene layers (back to front)
        draw_skyline();
        draw_road(frame);
        draw_street_lights(frame);
        draw_rain();
        draw_car(frame);
        draw_hud();
        
        // Lightning flash effect occasionally
        if (frame == 100 || frame == 180) {
            for (int y = 0; y < 12; y++) {
                for (int x = 0; x < SCREEN_WIDTH; x++) {
                    text40_putchar_color(x, y, ' ', COLOR_WHITE, COLOR_WHITE);
                }
            }
            display_flush();
            delay(500);
        }
        
        // Speed burst effect
        if (frame >= 150 && frame < 170) {
            // Motion blur lines
            for (int x = 0; x < SCREEN_WIDTH; x += 3) {
                text40_putchar_color(x, 15 + rand_range(5), '-', COLOR_BLUE, COLOR_BLACK);
            }
            text40_puts_color(15, 11, "TURBO", COLOR_YELLOW, COLOR_RED);
        }
        
        // Police sirens in distance (flashing red/blue)
        if (frame >= 200 && frame < 250) {
            unsigned char siren_color = (frame % 4 < 2) ? COLOR_RED : COLOR_BLUE;
            text40_putchar_color(2, 10, '*', siren_color, COLOR_BLACK);
            text40_putchar_color(37, 10, '*', siren_color, COLOR_BLACK);
        }
        
        display_flush();
        delay(2500); // Animation speed
        
        // Add new raindrops
        if (frame % 3 == 0) {
            for (int i = 0; i < MAX_RAINDROPS; i++) {
                if (!rain[i].active) {
                    rain[i].x = rand_range(SCREEN_WIDTH);
                    rain[i].y = 0;
                    rain[i].speed = 1 + rand_range(2);
                    rain[i].active = 1;
                    break;
                }
            }
        }
    }
    
    // End sequence
    text40_puts_color(14, 11, "DESTINATION", COLOR_WHITE, COLOR_BLUE);
    text40_puts_color(16, 12, "REACHED", COLOR_BLACK, COLOR_PINK);
    text40_puts_color(13, 14, "WELCOME TO", COLOR_GREEN, COLOR_BLACK);
    text40_puts_color(14, 15, "NEO BERLIN", COLOR_YELLOW, COLOR_BLACK);
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Cyberpunk drive complete!");
    
    return 0;
}