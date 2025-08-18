// Classic Galaga/Galaxian space shooter for TEXT40 display
// Features alien formations, diving attacks, player ship, and scoring

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Game dimensions
#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25
#define MAX_ALIENS 30
#define MAX_BULLETS 5
#define MAX_ALIEN_BULLETS 8
#define MAX_STARS 20
#define FORMATION_ROWS 5
#define FORMATION_COLS 8

// Game state
int player_x;
int player_lives;
int score;
int level;
int game_over;
int wave_cleared;
int aliens_remaining;
int frame_count;
int formation_x;
int formation_y;
int formation_dx;
int formation_dy;
int attack_timer;
int next_attacker;

// Alien types
#define ALIEN_BEE 0
#define ALIEN_BUTTERFLY 1
#define ALIEN_BOSS 2
#define ALIEN_EMPTY 3

// Alien structure
struct Alien {
    int x;          // Position in formation or diving
    int y;
    int type;
    int alive;
    int diving;     // Is this alien diving?
    int dive_x;     // Fixed point for smooth diving
    int dive_y;     // Fixed point for smooth diving
    int dive_vx;    // Dive velocity
    int dive_vy;
    int dive_pattern; // Which dive pattern to follow
    int formation_x; // Original formation position
    int formation_y;
};

// Bullet structure
struct Bullet {
    int x;
    int y;
    int active;
    int dy;  // Direction (-1 for player, 1 for alien)
};

// Star structure for background
struct Star {
    int x;
    int y;
    int speed;
};

struct Alien aliens[MAX_ALIENS];
struct Bullet player_bullets[MAX_BULLETS];
struct Bullet alien_bullets[MAX_ALIEN_BULLETS];
struct Star stars[MAX_STARS];

// Get random number in range
int rand_range(int max) {
    return (rng_get() % max);
}

// Initialize stars
void init_stars() {
    for (int i = 0; i < MAX_STARS; i++) {
        stars[i].x = rand_range(SCREEN_WIDTH);
        stars[i].y = rand_range(SCREEN_HEIGHT - 3);
        stars[i].speed = 1 + rand_range(3);
    }
}

// Update stars
void update_stars() {
    for (int i = 0; i < MAX_STARS; i++) {
        stars[i].y += 1;
        if (stars[i].y >= SCREEN_HEIGHT - 2) {
            stars[i].y = 1;
            stars[i].x = rand_range(SCREEN_WIDTH);
        }
    }
}

// Initialize alien formation
void init_formation(int level_num) {
    aliens_remaining = 0;
    formation_x = 5;
    formation_y = 3;
    formation_dx = 1;
    formation_dy = 0;
    
    for (int row = 0; row < FORMATION_ROWS; row++) {
        for (int col = 0; col < FORMATION_COLS; col++) {
            int idx = row * FORMATION_COLS + col;
            if (idx < MAX_ALIENS) {
                aliens[idx].formation_x = col * 3;
                aliens[idx].formation_y = row * 2;
                aliens[idx].x = formation_x + aliens[idx].formation_x;
                aliens[idx].y = formation_y + aliens[idx].formation_y;
                
                // Assign alien types based on row
                if (row == 0) {
                    aliens[idx].type = ALIEN_BOSS;
                } else if (row <= 2) {
                    aliens[idx].type = ALIEN_BUTTERFLY;
                } else {
                    aliens[idx].type = ALIEN_BEE;
                }
                
                // Some positions empty for variety
                if ((row + col + level_num) % 7 == 0 && row > 0) {
                    aliens[idx].alive = 0;
                } else {
                    aliens[idx].alive = 1;
                    aliens_remaining++;
                }
                
                aliens[idx].diving = 0;
                aliens[idx].dive_pattern = rand_range(3);
            }
        }
    }
}

// Initialize game
void init_game() {
    player_x = SCREEN_WIDTH / 2 - 1;
    player_lives = 3;
    score = 0;
    level = 1;
    game_over = 0;
    wave_cleared = 0;
    frame_count = 0;
    attack_timer = 60;
    next_attacker = 0;
    
    // Clear bullets
    for (int i = 0; i < MAX_BULLETS; i++) {
        player_bullets[i].active = 0;
    }
    for (int i = 0; i < MAX_ALIEN_BULLETS; i++) {
        alien_bullets[i].active = 0;
    }
    
    init_stars();
    init_formation(level);
}

// Move player
void move_player(int dx) {
    player_x += dx;
    if (player_x < 1) player_x = 1;
    if (player_x > SCREEN_WIDTH - 3) player_x = SCREEN_WIDTH - 3;
}

// Fire player bullet
void fire_player_bullet() {
    for (int i = 0; i < MAX_BULLETS; i++) {
        if (!player_bullets[i].active) {
            player_bullets[i].x = player_x + 1;
            player_bullets[i].y = SCREEN_HEIGHT - 4;
            player_bullets[i].active = 1;
            player_bullets[i].dy = -1;
            break;
        }
    }
}

// Fire alien bullet
void fire_alien_bullet(int x, int y) {
    for (int i = 0; i < MAX_ALIEN_BULLETS; i++) {
        if (!alien_bullets[i].active) {
            alien_bullets[i].x = x;
            alien_bullets[i].y = y;
            alien_bullets[i].active = 1;
            alien_bullets[i].dy = 1;
            break;
        }
    }
}

// Start dive attack
void start_dive_attack(int alien_idx) {
    if (alien_idx >= 0 && alien_idx < MAX_ALIENS && 
        aliens[alien_idx].alive && !aliens[alien_idx].diving) {
        
        aliens[alien_idx].diving = 1;
        aliens[alien_idx].dive_x = aliens[alien_idx].x * 256;
        aliens[alien_idx].dive_y = aliens[alien_idx].y * 256;
        
        // Choose dive pattern
        int pattern = aliens[alien_idx].dive_pattern;
        if (pattern == 0) {
            // Straight dive toward player
            aliens[alien_idx].dive_vx = ((player_x - aliens[alien_idx].x) * 256) / 30;
            aliens[alien_idx].dive_vy = 256;
        } else if (pattern == 1) {
            // Curved dive
            aliens[alien_idx].dive_vx = (rand_range(512) - 256);
            aliens[alien_idx].dive_vy = 200;
        } else {
            // Loop pattern
            aliens[alien_idx].dive_vx = 300;
            aliens[alien_idx].dive_vy = 150;
        }
    }
}

// Update formation movement
void update_formation() {
    // Move formation
    if (frame_count % 30 == 0) {
        formation_x += formation_dx;
        
        // Check bounds and reverse
        int leftmost = formation_x;
        int rightmost = formation_x + FORMATION_COLS * 3;
        
        if (leftmost <= 1 || rightmost >= SCREEN_WIDTH - 1) {
            formation_dx = -formation_dx;
            formation_y += 1;
        }
    }
    
    // Update non-diving aliens to follow formation
    for (int i = 0; i < MAX_ALIENS; i++) {
        if (aliens[i].alive && !aliens[i].diving) {
            aliens[i].x = formation_x + aliens[i].formation_x;
            aliens[i].y = formation_y + aliens[i].formation_y;
        }
    }
}

// Update diving aliens
void update_diving_aliens() {
    for (int i = 0; i < MAX_ALIENS; i++) {
        if (aliens[i].alive && aliens[i].diving) {
            // Update dive position
            aliens[i].dive_x += aliens[i].dive_vx;
            aliens[i].dive_y += aliens[i].dive_vy;
            
            // Loop pattern adjustment
            if (aliens[i].dive_pattern == 2) {
                aliens[i].dive_vx -= 10;  // Curve back
            }
            
            aliens[i].x = aliens[i].dive_x / 256;
            aliens[i].y = aliens[i].dive_y / 256;
            
            // Return to formation if off screen
            if (aliens[i].x < 0 || aliens[i].x >= SCREEN_WIDTH ||
                aliens[i].y >= SCREEN_HEIGHT - 2) {
                
                if (aliens[i].y >= SCREEN_HEIGHT - 2) {
                    // Wrap to top and return to formation
                    aliens[i].diving = 0;
                    aliens[i].x = formation_x + aliens[i].formation_x;
                    aliens[i].y = formation_y + aliens[i].formation_y;
                }
                
                // Wrap horizontally
                if (aliens[i].x < 0) aliens[i].x = SCREEN_WIDTH - 1;
                if (aliens[i].x >= SCREEN_WIDTH) aliens[i].x = 0;
                aliens[i].dive_x = aliens[i].x * 256;
            }
            
            // Occasionally fire while diving
            if (rand_range(100) < 5) {
                fire_alien_bullet(aliens[i].x, aliens[i].y + 1);
            }
        }
    }
}

// Update bullets
void update_bullets() {
    // Player bullets
    for (int i = 0; i < MAX_BULLETS; i++) {
        if (player_bullets[i].active) {
            player_bullets[i].y += player_bullets[i].dy;
            
            if (player_bullets[i].y < 1) {
                player_bullets[i].active = 0;
            }
            
            // Check alien hits
            for (int j = 0; j < MAX_ALIENS; j++) {
                if (aliens[j].alive) {
                    if (player_bullets[i].x >= aliens[j].x &&
                        player_bullets[i].x <= aliens[j].x + 1 &&
                        player_bullets[i].y == aliens[j].y) {
                        
                        // Hit!
                        aliens[j].alive = 0;
                        player_bullets[i].active = 0;
                        aliens_remaining--;
                        
                        // Score based on type
                        if (aliens[j].type == ALIEN_BOSS) {
                            score += 150;
                        } else if (aliens[j].type == ALIEN_BUTTERFLY) {
                            score += 80;
                        } else {
                            score += 50;
                        }
                        
                        // Bonus for diving alien
                        if (aliens[j].diving) {
                            score += 100;
                        }
                        
                        break;
                    }
                }
            }
        }
    }
    
    // Alien bullets
    for (int i = 0; i < MAX_ALIEN_BULLETS; i++) {
        if (alien_bullets[i].active) {
            alien_bullets[i].y += alien_bullets[i].dy;
            
            if (alien_bullets[i].y >= SCREEN_HEIGHT - 1) {
                alien_bullets[i].active = 0;
            }
            
            // Check player hit
            if (alien_bullets[i].y == SCREEN_HEIGHT - 3 &&
                alien_bullets[i].x >= player_x &&
                alien_bullets[i].x <= player_x + 2) {
                
                alien_bullets[i].active = 0;
                player_lives--;
                
                if (player_lives <= 0) {
                    game_over = 1;
                }
            }
        }
    }
}

// Update game
void update_game() {
    frame_count++;
    
    // Update formations
    update_formation();
    update_diving_aliens();
    update_bullets();
    update_stars();
    
    // Launch dive attacks
    attack_timer--;
    if (attack_timer <= 0) {
        attack_timer = 60 + rand_range(60);
        
        // Choose random alive alien to attack
        int attempts = 0;
        while (attempts < 10) {
            int idx = rand_range(MAX_ALIENS);
            if (aliens[idx].alive && !aliens[idx].diving) {
                start_dive_attack(idx);
                break;
            }
            attempts++;
        }
    }
    
    // Random alien shooting from formation
    if (rand_range(100) < 3) {
        int idx = rand_range(MAX_ALIENS);
        if (aliens[idx].alive && !aliens[idx].diving) {
            fire_alien_bullet(aliens[idx].x, aliens[idx].y + 1);
        }
    }
    
    // Check wave cleared
    if (aliens_remaining == 0) {
        level++;
        init_formation(level);
    }
}

// Draw game
void draw_game() {
    // Clear screen
    display_clear();
    
    // Draw stars
    for (int i = 0; i < MAX_STARS; i++) {
        char star_char = (stars[i].speed == 1) ? '.' : 
                        (stars[i].speed == 2) ? '+' : '*';
        text40_putchar_color(stars[i].x, stars[i].y, star_char, 
                           COLOR_DARK_GRAY, COLOR_BLACK);
    }
    
    // Draw aliens
    for (int i = 0; i < MAX_ALIENS; i++) {
        if (aliens[i].alive) {
            char alien_char;
            unsigned char alien_color;
            
            if (aliens[i].type == ALIEN_BOSS) {
                alien_char = 'W';
                alien_color = COLOR_RED;
            } else if (aliens[i].type == ALIEN_BUTTERFLY) {
                alien_char = 'M';
                alien_color = COLOR_YELLOW;
            } else {
                alien_char = 'v';
                alien_color = COLOR_GREEN;
            }
            
            // Flashing if diving
            if (aliens[i].diving && frame_count % 4 < 2) {
                alien_color = COLOR_WHITE;
            }
            
            text40_putchar_color(aliens[i].x, aliens[i].y, alien_char,
                               alien_color, COLOR_BLACK);
            
            // Wings for larger aliens
            if (aliens[i].type == ALIEN_BOSS || aliens[i].type == ALIEN_BUTTERFLY) {
                if (aliens[i].x > 0) {
                    text40_putchar_color(aliens[i].x - 1, aliens[i].y, 
                                       (frame_count % 8 < 4) ? '\\' : '/', 
                                       alien_color, COLOR_BLACK);
                }
                if (aliens[i].x < SCREEN_WIDTH - 1) {
                    text40_putchar_color(aliens[i].x + 1, aliens[i].y, 
                                       (frame_count % 8 < 4) ? '/' : '\\', 
                                       alien_color, COLOR_BLACK);
                }
            }
        }
    }
    
    // Draw player ship
    text40_putchar_color(player_x, SCREEN_HEIGHT - 3, '/', 
                       COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(player_x + 1, SCREEN_HEIGHT - 3, 'A', 
                       COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(player_x + 2, SCREEN_HEIGHT - 3, '\\', 
                       COLOR_WHITE, COLOR_BLACK);
    
    // Engine
    text40_putchar_color(player_x + 1, SCREEN_HEIGHT - 2, 
                       (frame_count % 4 < 2) ? 'v' : 'V',
                       COLOR_ORANGE, COLOR_BLACK);
    
    // Draw bullets
    for (int i = 0; i < MAX_BULLETS; i++) {
        if (player_bullets[i].active) {
            text40_putchar_color(player_bullets[i].x, player_bullets[i].y,
                               '|', COLOR_YELLOW, COLOR_BLACK);
        }
    }
    
    for (int i = 0; i < MAX_ALIEN_BULLETS; i++) {
        if (alien_bullets[i].active) {
            text40_putchar_color(alien_bullets[i].x, alien_bullets[i].y,
                               '!', COLOR_RED, COLOR_BLACK);
        }
    }
    
    // Draw UI
    // Top bar
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, 0, '=', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Score
    text40_puts_color(1, 0, "SCORE:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 5; i >= 0; i--) {
        int digit = score;
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(8 + (5 - i), 0, '0' + (digit % 10), 
                           COLOR_YELLOW, COLOR_BLACK);
    }
    
    // Level
    text40_puts_color(16, 0, "WAVE:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(22, 0, '0' + level, COLOR_GREEN, COLOR_BLACK);
    
    // Lives
    text40_puts_color(26, 0, "SHIPS:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 0; i < player_lives && i < 5; i++) {
        text40_putchar_color(33 + i, 0, 'A', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Bottom bar
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, SCREEN_HEIGHT - 1, '=', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Stage cleared message
    if (aliens_remaining == 0) {
        text40_puts_color(13, 12, " STAGE CLEAR! ", COLOR_BLACK, COLOR_YELLOW);
    }
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Get demo input
char get_demo_input() {
    // Simple AI
    if (rand_range(100) < 30) {
        // Find nearest diving alien
        int nearest_x = -1;
        int min_dist = 999;
        
        for (int i = 0; i < MAX_ALIENS; i++) {
            if (aliens[i].alive && aliens[i].diving) {
                int dist = (aliens[i].x - player_x) * (aliens[i].x - player_x);
                if (dist < min_dist) {
                    min_dist = dist;
                    nearest_x = aliens[i].x;
                }
            }
        }
        
        // Move toward threat
        if (nearest_x >= 0) {
            if (nearest_x < player_x - 1) return 'a';
            if (nearest_x > player_x + 1) return 'd';
        }
    }
    
    // Shoot occasionally
    if (rand_range(100) < 20) {
        return ' ';
    }
    
    // Random movement
    if (rand_range(100) < 10) {
        return (rand_range(2) == 0) ? 'a' : 'd';
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
    int max_frames = 1500;
    
    while (!game_over && frame_count < max_frames) {
        // Get input (demo AI)
        char input = get_demo_input();
        
        if (input == 'a') {
            move_player(-1);
        } else if (input == 'd') {
            move_player(1);
        } else if (input == ' ') {
            fire_player_bullet();
        }
        
        // Update game
        update_game();
        
        // Draw everything
        draw_game();
        display_flush();
        
        // Frame delay
        delay(4000);
    }
    
    // Game over screen
    if (game_over) {
        text40_puts_color(13, 11, " GAME OVER ", COLOR_WHITE, COLOR_RED);
    } else {
        text40_puts_color(11, 11, " DEMO COMPLETE ", COLOR_WHITE, COLOR_GREEN);
    }
    
    text40_puts_color(12, 13, " FINAL SCORE ", COLOR_BLACK, COLOR_YELLOW);
    for (int i = 5; i >= 0; i--) {
        int digit = score;
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(14 + (5 - i), 14, '0' + (digit % 10), 
                           COLOR_YELLOW, COLOR_BLACK);
    }
    
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Galaga demo completed!");
    
    return 0;
}