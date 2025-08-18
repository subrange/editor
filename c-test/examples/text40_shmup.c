// Space Shoot 'em Up for TEXT40 display - Simplified version
// No structs, no switch, no typedef - only arrays and basic control flow

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Game dimensions
#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25
#define MAX_BULLETS 20
#define MAX_ENEMIES 12
#define MAX_ENEMY_BULLETS 15
#define MAX_PARTICLES 30
#define MAX_POWERUPS 3
#define MAX_STARS 20

// Game states
#define STATE_TITLE 0
#define STATE_PLAYING 1
#define STATE_GAMEOVER 2
#define STATE_WIN 3

// Enemy types
#define ENEMY_BASIC 0
#define ENEMY_FAST 1
#define ENEMY_HEAVY 2
#define ENEMY_BOSS 3

// Power-up types
#define POWERUP_RAPID 0
#define POWERUP_SPREAD 1
#define POWERUP_SHIELD 2
#define POWERUP_LASER 3

// Player data (using separate arrays instead of struct)
int player_x;
int player_y;
int player_lives;
int player_shield;
int player_power_timer;
int player_power_type;
int player_invulnerable;

// Bullet arrays (instead of struct)
int bullet_x[MAX_BULLETS];
int bullet_y[MAX_BULLETS];
int bullet_vx[MAX_BULLETS];
int bullet_vy[MAX_BULLETS];
int bullet_active[MAX_BULLETS];
int bullet_is_laser[MAX_BULLETS];

// Enemy bullet arrays
int enemy_bullet_x[MAX_ENEMY_BULLETS];
int enemy_bullet_y[MAX_ENEMY_BULLETS];
int enemy_bullet_vx[MAX_ENEMY_BULLETS];
int enemy_bullet_vy[MAX_ENEMY_BULLETS];
int enemy_bullet_active[MAX_ENEMY_BULLETS];

// Enemy arrays
int enemy_x[MAX_ENEMIES];
int enemy_y[MAX_ENEMIES];
int enemy_vx[MAX_ENEMIES];
int enemy_type[MAX_ENEMIES];
int enemy_hp[MAX_ENEMIES];
int enemy_active[MAX_ENEMIES];
int enemy_shoot_timer[MAX_ENEMIES];
int enemy_pattern_timer[MAX_ENEMIES];

// Particle arrays
int particle_x[MAX_PARTICLES];
int particle_y[MAX_PARTICLES];
int particle_vx[MAX_PARTICLES];
int particle_vy[MAX_PARTICLES];
int particle_life[MAX_PARTICLES];
unsigned char particle_color[MAX_PARTICLES];
char particle_symbol[MAX_PARTICLES];

// Power-up arrays
int powerup_x[MAX_POWERUPS];
int powerup_y[MAX_POWERUPS];
int powerup_type[MAX_POWERUPS];
int powerup_active[MAX_POWERUPS];

// Star arrays for background
int star_x[MAX_STARS];
int star_y[MAX_STARS];
int star_speed[MAX_STARS];

// Game globals
int score;
int high_score = 0;
int wave;
int enemies_remaining;
int game_state;
int frame_count;
int boss_spawned;
int combo_counter;
int combo_timer;

// Get random number in range
int rand_range(int max) {
    return (rng_get() % max);
}

// Initialize stars
void init_stars() {
    for (int i = 0; i < MAX_STARS; i++) {
        star_x[i] = rand_range(SCREEN_WIDTH);
        star_y[i] = rand_range(SCREEN_HEIGHT);
        star_speed[i] = 1 + rand_range(3);
    }
}

// Update stars
void update_stars() {
    for (int i = 0; i < MAX_STARS; i++) {
        star_y[i] += star_speed[i];
        if (star_y[i] >= SCREEN_HEIGHT) {
            star_y[i] = 0;
            star_x[i] = rand_range(SCREEN_WIDTH);
        }
    }
}

// Draw stars
void draw_stars() {
    for (int i = 0; i < MAX_STARS; i++) {
        unsigned char color = COLOR_DARK_GRAY;
        char symbol = '.';
        
        if (star_speed[i] == 2) {
            color = COLOR_LIGHT_GRAY;
        } else if (star_speed[i] == 3) {
            color = COLOR_WHITE;
            symbol = '+';
        }
        
        text40_putchar_color(star_x[i], star_y[i], symbol, color, COLOR_BLACK);
    }
}

// Create particle effect
void spawn_particle(int x, int y, unsigned char color, char symbol) {
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] <= 0) {
            particle_x[i] = x;
            particle_y[i] = y;
            particle_vx[i] = rand_range(5) - 2;
            particle_vy[i] = rand_range(3) - 1;
            particle_life[i] = 10 + rand_range(10);
            particle_color[i] = color;
            particle_symbol[i] = symbol;
            break;
        }
    }
}

// Spawn explosion
void spawn_explosion(int x, int y, int size) {
    for (int i = 0; i < size; i++) {
        spawn_particle(x, y, COLOR_ORANGE, '*');
        spawn_particle(x, y, COLOR_YELLOW, '.');
        spawn_particle(x, y, COLOR_RED, 'o');
    }
}

// Initialize player
void init_player() {
    player_x = SCREEN_WIDTH / 2;
    player_y = SCREEN_HEIGHT - 4;
    player_lives = 3;
    player_shield = 0;
    player_power_timer = 0;
    player_power_type = -1;
    player_invulnerable = 0;
}

// Initialize game
void init_game() {
    init_player();
    init_stars();
    
    score = 0;
    wave = 1;
    enemies_remaining = 0;
    game_state = STATE_TITLE;
    frame_count = 0;
    boss_spawned = 0;
    combo_counter = 0;
    combo_timer = 0;
    
    // Clear all entities
    for (int i = 0; i < MAX_BULLETS; i++) {
        bullet_active[i] = 0;
    }
    for (int i = 0; i < MAX_ENEMY_BULLETS; i++) {
        enemy_bullet_active[i] = 0;
    }
    for (int i = 0; i < MAX_ENEMIES; i++) {
        enemy_active[i] = 0;
    }
    for (int i = 0; i < MAX_PARTICLES; i++) {
        particle_life[i] = 0;
    }
    for (int i = 0; i < MAX_POWERUPS; i++) {
        powerup_active[i] = 0;
    }
}

// Spawn enemy wave
void spawn_wave(int wave_num) {
    enemies_remaining = 0;
    
    if (wave_num % 5 == 0 && !boss_spawned) {
        // Boss wave
        enemy_x[0] = SCREEN_WIDTH / 2;
        enemy_y[0] = 3;
        enemy_vx[0] = 1;
        enemy_type[0] = ENEMY_BOSS;
        enemy_hp[0] = 20 + wave_num * 2;
        enemy_active[0] = 1;
        enemy_shoot_timer[0] = 0;
        enemy_pattern_timer[0] = 0;
        enemies_remaining = 1;
        boss_spawned = 1;
    } else {
        // Regular wave
        boss_spawned = 0;
        int enemy_count = 4 + wave_num;
        if (enemy_count > MAX_ENEMIES) enemy_count = MAX_ENEMIES;
        
        for (int i = 0; i < enemy_count; i++) {
            enemy_x[i] = 5 + (i % 6) * 5;
            enemy_y[i] = 2 + (i / 6) * 2;
            enemy_vx[i] = (rand_range(2) == 0) ? 1 : -1;
            
            // Enemy type based on wave
            if (wave_num > 3 && rand_range(100) < 30) {
                enemy_type[i] = ENEMY_HEAVY;
                enemy_hp[i] = 3;
            } else if (wave_num > 2 && rand_range(100) < 40) {
                enemy_type[i] = ENEMY_FAST;
                enemy_hp[i] = 1;
            } else {
                enemy_type[i] = ENEMY_BASIC;
                enemy_hp[i] = 2;
            }
            
            enemy_active[i] = 1;
            enemy_shoot_timer[i] = rand_range(60);
            enemy_pattern_timer[i] = 0;
            enemies_remaining++;
        }
    }
}

// Fire player bullet
void fire_bullet(int spread) {
    if (player_power_type == POWERUP_LASER) {
        // Laser beam
        for (int i = 0; i < MAX_BULLETS; i++) {
            if (!bullet_active[i]) {
                bullet_x[i] = player_x;
                bullet_y[i] = player_y - 1;
                bullet_vx[i] = 0;
                bullet_vy[i] = -2;
                bullet_active[i] = 1;
                bullet_is_laser[i] = 1;
                break;
            }
        }
    } else if (spread || player_power_type == POWERUP_SPREAD) {
        // Spread shot - fire three bullets
        int angles[3];
        angles[0] = -1;
        angles[1] = 0;
        angles[2] = 1;
        
        for (int j = 0; j < 3; j++) {
            for (int i = 0; i < MAX_BULLETS; i++) {
                if (!bullet_active[i]) {
                    bullet_x[i] = player_x;
                    bullet_y[i] = player_y - 1;
                    bullet_vx[i] = angles[j];
                    bullet_vy[i] = -1;
                    bullet_active[i] = 1;
                    bullet_is_laser[i] = 0;
                    break;
                }
            }
        }
    } else {
        // Normal shot
        for (int i = 0; i < MAX_BULLETS; i++) {
            if (!bullet_active[i]) {
                bullet_x[i] = player_x;
                bullet_y[i] = player_y - 1;
                bullet_vx[i] = 0;
                bullet_vy[i] = -1;
                bullet_active[i] = 1;
                bullet_is_laser[i] = 0;
                break;
            }
        }
    }
}

// Enemy shoots
void enemy_shoot(int idx) {
    for (int i = 0; i < MAX_ENEMY_BULLETS; i++) {
        if (!enemy_bullet_active[i]) {
            enemy_bullet_x[i] = enemy_x[idx];
            enemy_bullet_y[i] = enemy_y[idx] + 1;
            
            if (enemy_type[idx] == ENEMY_BOSS) {
                // Boss fires spread pattern
                enemy_bullet_vx[i] = rand_range(3) - 1;
                enemy_bullet_vy[i] = 1;
            } else if (enemy_type[idx] == ENEMY_FAST) {
                // Fast enemy aims at player
                int dx = player_x - enemy_x[idx];
                if (dx > 0) {
                    enemy_bullet_vx[i] = 1;
                } else if (dx < 0) {
                    enemy_bullet_vx[i] = -1;
                } else {
                    enemy_bullet_vx[i] = 0;
                }
                enemy_bullet_vy[i] = 1;
            } else {
                // Basic downward shot
                enemy_bullet_vx[i] = 0;
                enemy_bullet_vy[i] = 1;
            }
            
            enemy_bullet_active[i] = 1;
            break;
        }
    }
}

// Spawn power-up
void spawn_powerup(int x, int y) {
    if (rand_range(100) < 25) { // 25% chance
        for (int i = 0; i < MAX_POWERUPS; i++) {
            if (!powerup_active[i]) {
                powerup_x[i] = x;
                powerup_y[i] = y;
                powerup_type[i] = rand_range(4);
                powerup_active[i] = 1;
                break;
            }
        }
    }
}

// Update bullets
void update_bullets() {
    // Player bullets
    for (int i = 0; i < MAX_BULLETS; i++) {
        if (!bullet_active[i]) continue;
        
        bullet_x[i] += bullet_vx[i];
        bullet_y[i] += bullet_vy[i];
        
        // Remove if off screen
        if (bullet_y[i] < 0 || bullet_x[i] < 0 || bullet_x[i] >= SCREEN_WIDTH) {
            bullet_active[i] = 0;
            continue;
        }
        
        // Check enemy collisions
        for (int j = 0; j < MAX_ENEMIES; j++) {
            if (!enemy_active[j]) continue;
            
            int hit = 0;
            if (bullet_is_laser[i]) {
                // Laser hits everything in column
                if (bullet_x[i] == enemy_x[j] && bullet_y[i] <= enemy_y[j]) {
                    hit = 1;
                }
            } else {
                // Normal bullet collision
                if (bullet_x[i] == enemy_x[j] && bullet_y[i] == enemy_y[j]) {
                    hit = 1;
                    bullet_active[i] = 0;
                }
            }
            
            if (hit) {
                enemy_hp[j]--;
                spawn_particle(enemy_x[j], enemy_y[j], COLOR_YELLOW, '*');
                
                if (enemy_hp[j] <= 0) {
                    // Enemy destroyed
                    enemy_active[j] = 0;
                    enemies_remaining--;
                    
                    // Score and combo
                    int points = 10;
                    if (enemy_type[j] == ENEMY_BOSS) {
                        points = 500;
                    } else if (enemy_type[j] == ENEMY_HEAVY) {
                        points = 50;
                    } else if (enemy_type[j] == ENEMY_FAST) {
                        points = 30;
                    }
                    
                    if (combo_timer > 0) {
                        combo_counter++;
                        points *= combo_counter;
                    } else {
                        combo_counter = 1;
                    }
                    combo_timer = 30;
                    
                    score += points;
                    
                    // Explosion effect
                    int explosion_size = 2;
                    if (enemy_type[j] == ENEMY_BOSS) {
                        explosion_size = 5;
                    }
                    spawn_explosion(enemy_x[j], enemy_y[j], explosion_size);
                    
                    // Spawn power-up
                    spawn_powerup(enemy_x[j], enemy_y[j]);
                }
            }
        }
    }
    
    // Enemy bullets
    for (int i = 0; i < MAX_ENEMY_BULLETS; i++) {
        if (!enemy_bullet_active[i]) continue;
        
        enemy_bullet_x[i] += enemy_bullet_vx[i];
        enemy_bullet_y[i] += enemy_bullet_vy[i];
        
        // Remove if off screen
        if (enemy_bullet_y[i] >= SCREEN_HEIGHT || enemy_bullet_x[i] < 0 || 
            enemy_bullet_x[i] >= SCREEN_WIDTH) {
            enemy_bullet_active[i] = 0;
            continue;
        }
        
        // Check player collision
        if (player_invulnerable <= 0 &&
            enemy_bullet_x[i] == player_x && enemy_bullet_y[i] == player_y) {
            
            if (player_shield > 0) {
                player_shield--;
                spawn_particle(player_x, player_y, COLOR_BLUE, 'o');
            } else {
                player_lives--;
                player_invulnerable = 60;
                spawn_explosion(player_x, player_y, 3);
                
                if (player_lives <= 0) {
                    game_state = STATE_GAMEOVER;
                }
            }
            
            enemy_bullet_active[i] = 0;
        }
    }
}

// Update enemies
void update_enemies() {
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (!enemy_active[i]) continue;
        
        enemy_pattern_timer[i]++;
        
        // Movement patterns
        if (enemy_type[i] == ENEMY_BOSS) {
            // Boss movement - sine wave
            enemy_x[i] += enemy_vx[i];
            if (enemy_x[i] <= 5 || enemy_x[i] >= SCREEN_WIDTH - 5) {
                enemy_vx[i] = -enemy_vx[i];
            }
            
            // Boss shoots more frequently
            enemy_shoot_timer[i]++;
            if (enemy_shoot_timer[i] > 20) {
                enemy_shoot(i);
                enemy_shoot_timer[i] = 0;
            }
        } else if (enemy_type[i] == ENEMY_FAST) {
            // Fast enemy - zigzag movement
            if (enemy_pattern_timer[i] % 20 == 0) {
                enemy_vx[i] = rand_range(3) - 1;
            }
            enemy_x[i] += enemy_vx[i];
            
            if (enemy_pattern_timer[i] % 40 == 0) {
                enemy_y[i]++;
            }
        } else {
            // Basic enemy - horizontal movement
            enemy_x[i] += enemy_vx[i];
            if (enemy_x[i] <= 1 || enemy_x[i] >= SCREEN_WIDTH - 2) {
                enemy_vx[i] = -enemy_vx[i];
                enemy_y[i]++;
            }
        }
        
        // Boundary checks
        if (enemy_x[i] < 0) enemy_x[i] = 0;
        if (enemy_x[i] >= SCREEN_WIDTH) enemy_x[i] = SCREEN_WIDTH - 1;
        
        // Game over if enemy reaches bottom
        if (enemy_y[i] >= SCREEN_HEIGHT - 3) {
            game_state = STATE_GAMEOVER;
        }
        
        // Enemy shooting
        enemy_shoot_timer[i]++;
        int shoot_rate = 100;
        if (enemy_type[i] == ENEMY_HEAVY) {
            shoot_rate = 80;
        } else if (enemy_type[i] == ENEMY_FAST) {
            shoot_rate = 60;
        }
        
        if (enemy_shoot_timer[i] > shoot_rate) {
            enemy_shoot(i);
            enemy_shoot_timer[i] = 0;
        }
    }
}

// Update particles
void update_particles() {
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] > 0) {
            particle_x[i] += particle_vx[i];
            particle_y[i] += particle_vy[i];
            particle_life[i]--;
            
            // Fade effect
            if (particle_life[i] < 5 && particle_color[i] == COLOR_ORANGE) {
                particle_color[i] = COLOR_RED;
            }
        }
    }
}

// Update power-ups
void update_powerups() {
    for (int i = 0; i < MAX_POWERUPS; i++) {
        if (!powerup_active[i]) continue;
        
        powerup_y[i]++;
        
        // Remove if off screen
        if (powerup_y[i] >= SCREEN_HEIGHT) {
            powerup_active[i] = 0;
            continue;
        }
        
        // Check player collision
        if (powerup_x[i] == player_x && powerup_y[i] == player_y) {
            // Apply power-up based on type
            if (powerup_type[i] == POWERUP_RAPID) {
                player_power_type = POWERUP_RAPID;
                player_power_timer = 300;
                score += 50;
            } else if (powerup_type[i] == POWERUP_SPREAD) {
                player_power_type = POWERUP_SPREAD;
                player_power_timer = 200;
                score += 50;
            } else if (powerup_type[i] == POWERUP_SHIELD) {
                player_shield = 3;
                score += 100;
            } else if (powerup_type[i] == POWERUP_LASER) {
                player_power_type = POWERUP_LASER;
                player_power_timer = 150;
                score += 75;
            }
            
            powerup_active[i] = 0;
            spawn_particle(player_x, player_y, COLOR_GREEN, '+');
        }
    }
}

// Move player
void move_player(int dx, int dy) {
    player_x += dx;
    player_y += dy;
    
    // Boundaries
    if (player_x < 1) player_x = 1;
    if (player_x >= SCREEN_WIDTH - 1) player_x = SCREEN_WIDTH - 2;
    if (player_y < SCREEN_HEIGHT / 2) player_y = SCREEN_HEIGHT / 2;
    if (player_y >= SCREEN_HEIGHT - 2) player_y = SCREEN_HEIGHT - 3;
}

// Draw game
void draw_game() {
    // Clear screen
    display_clear();
    
    // Draw starfield background
    draw_stars();
    
    // Draw particles (background layer)
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] > 0 && particle_life[i] < 8) {
            if (particle_x[i] >= 0 && particle_x[i] < SCREEN_WIDTH &&
                particle_y[i] >= 0 && particle_y[i] < SCREEN_HEIGHT) {
                text40_putchar_color(particle_x[i], particle_y[i], 
                                   particle_symbol[i], particle_color[i], COLOR_BLACK);
            }
        }
    }
    
    // Draw enemies
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (!enemy_active[i]) continue;
        
        if (enemy_type[i] == ENEMY_BOSS) {
            // Boss sprite - using individual chars to avoid string issues
            text40_puts_color(enemy_x[i] - 2, enemy_y[i] - 1, "<-o->", COLOR_RED, COLOR_BLACK);
            // Middle row - draw each char separately
            text40_putchar_color(enemy_x[i] - 2, enemy_y[i], '\\', COLOR_DARK_PURPLE, COLOR_BLACK);
            text40_putchar_color(enemy_x[i] - 1, enemy_y[i], '\\', COLOR_DARK_PURPLE, COLOR_BLACK);
            text40_putchar_color(enemy_x[i], enemy_y[i], 'V', COLOR_DARK_PURPLE, COLOR_BLACK);
            text40_putchar_color(enemy_x[i] + 1, enemy_y[i], '/', COLOR_DARK_PURPLE, COLOR_BLACK);
            text40_putchar_color(enemy_x[i] + 2, enemy_y[i], '/', COLOR_DARK_PURPLE, COLOR_BLACK);
            text40_putchar_color(enemy_x[i], enemy_y[i] + 1, 'W', COLOR_RED, COLOR_BLACK);
        } else if (enemy_type[i] == ENEMY_HEAVY) {
            text40_putchar_color(enemy_x[i], enemy_y[i], 'H', COLOR_ORANGE, COLOR_BLACK);
        } else if (enemy_type[i] == ENEMY_FAST) {
            text40_putchar_color(enemy_x[i], enemy_y[i], 'v', COLOR_YELLOW, COLOR_BLACK);
        } else {
            text40_putchar_color(enemy_x[i], enemy_y[i], 'V', COLOR_GREEN, COLOR_BLACK);
        }
    }
    
    // Draw power-ups
    for (int i = 0; i < MAX_POWERUPS; i++) {
        if (!powerup_active[i]) continue;
        
        char symbol = '?';
        unsigned char color = COLOR_WHITE;
        
        if (powerup_type[i] == POWERUP_RAPID) {
            symbol = 'R';
            color = COLOR_YELLOW;
        } else if (powerup_type[i] == POWERUP_SPREAD) {
            symbol = 'S';
            color = COLOR_BLUE;
        } else if (powerup_type[i] == POWERUP_SHIELD) {
            symbol = 'D';
            color = COLOR_GREEN;
        } else if (powerup_type[i] == POWERUP_LASER) {
            symbol = 'L';
            color = COLOR_PINK;
        }
        
        text40_putchar_color(powerup_x[i], powerup_y[i], symbol, color, COLOR_BLACK);
    }
    
    // Draw bullets
    for (int i = 0; i < MAX_BULLETS; i++) {
        if (!bullet_active[i]) continue;
        
        if (bullet_is_laser[i]) {
            // Draw laser beam
            for (int y = bullet_y[i]; y >= 0; y--) {
                text40_putchar_color(bullet_x[i], y, '|', COLOR_PINK, COLOR_BLACK);
            }
        } else {
            text40_putchar_color(bullet_x[i], bullet_y[i], '|', COLOR_YELLOW, COLOR_BLACK);
        }
    }
    
    // Draw enemy bullets
    for (int i = 0; i < MAX_ENEMY_BULLETS; i++) {
        if (!enemy_bullet_active[i]) continue;
        text40_putchar_color(enemy_bullet_x[i], enemy_bullet_y[i], 'o', COLOR_RED, COLOR_BLACK);
    }
    
    // Draw player ship
    if (player_invulnerable <= 0 || frame_count % 4 < 2) {
        unsigned char ship_color = COLOR_WHITE;
        if (player_shield > 0) {
            ship_color = COLOR_BLUE;
        }
        
        text40_putchar_color(player_x, player_y, 'A', ship_color, COLOR_BLACK);
        
        // Ship wings
        if (player_x > 0) {
            text40_putchar_color(player_x - 1, player_y, '/', ship_color, COLOR_BLACK);
        }
        if (player_x < SCREEN_WIDTH - 1) {
            text40_putchar_color(player_x + 1, player_y, '\\', ship_color, COLOR_BLACK);
        }
        
        // Shield indicator
        if (player_shield > 0) {
            text40_putchar_color(player_x, player_y - 1, '-', COLOR_BLUE, COLOR_BLACK);
            text40_putchar_color(player_x, player_y + 1, '-', COLOR_BLUE, COLOR_BLACK);
        }
    }
    
    // Draw particles (foreground layer)
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (particle_life[i] > 0 && particle_life[i] >= 8) {
            if (particle_x[i] >= 0 && particle_x[i] < SCREEN_WIDTH &&
                particle_y[i] >= 0 && particle_y[i] < SCREEN_HEIGHT) {
                text40_putchar_color(particle_x[i], particle_y[i], 
                                   particle_symbol[i], particle_color[i], COLOR_BLACK);
            }
        }
    }
    
    // Draw HUD
    draw_hud();
}

// Draw HUD
void draw_hud() {
    // Top border
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, 0, '=', COLOR_DARK_BLUE, COLOR_BLACK);
    }
    
    // Score
    text40_puts_color(1, 0, "SCORE:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 5; i >= 0; i--) {
        int digit = score;
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(8 + (5 - i), 0, '0' + (digit % 10), COLOR_YELLOW, COLOR_BLACK);
    }
    
    // Wave
    text40_puts_color(16, 0, "WAVE:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(22, 0, '0' + (wave / 10), COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(23, 0, '0' + (wave % 10), COLOR_GREEN, COLOR_BLACK);
    
    // Lives
    text40_puts_color(26, 0, "LIVES:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 0; i < player_lives && i < 5; i++) {
        text40_putchar_color(33 + i, 0, 'A', COLOR_RED, COLOR_BLACK);
    }
    
    // Bottom status bar
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, SCREEN_HEIGHT - 1, '=', COLOR_DARK_BLUE, COLOR_BLACK);
    }
    
    // Power-up indicator
    if (player_power_timer > 0) {
        if (player_power_type == POWERUP_RAPID) {
            text40_puts_color(1, SCREEN_HEIGHT - 1, "RAPID", COLOR_YELLOW, COLOR_BLACK);
        } else if (player_power_type == POWERUP_SPREAD) {
            text40_puts_color(1, SCREEN_HEIGHT - 1, "SPREAD", COLOR_BLUE, COLOR_BLACK);
        } else if (player_power_type == POWERUP_LASER) {
            text40_puts_color(1, SCREEN_HEIGHT - 1, "LASER", COLOR_PINK, COLOR_BLACK);
        }
    }
    
    // Shield indicator
    if (player_shield > 0) {
        text40_puts_color(10, SCREEN_HEIGHT - 1, "SHIELD:", COLOR_GREEN, COLOR_BLACK);
        for (int i = 0; i < player_shield; i++) {
            text40_putchar_color(18 + i, SCREEN_HEIGHT - 1, 'o', COLOR_GREEN, COLOR_BLACK);
        }
    }
    
    // Combo indicator
    if (combo_timer > 0) {
        text40_puts_color(25, SCREEN_HEIGHT - 1, "COMBO x", COLOR_ORANGE, COLOR_BLACK);
        text40_putchar_color(32, SCREEN_HEIGHT - 1, '0' + combo_counter, COLOR_YELLOW, COLOR_BLACK);
    }
    
    // Enemies remaining
    text40_puts_color(35, SCREEN_HEIGHT - 1, "E:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(37, SCREEN_HEIGHT - 1, '0' + (enemies_remaining / 10), COLOR_RED, COLOR_BLACK);
    text40_putchar_color(38, SCREEN_HEIGHT - 1, '0' + (enemies_remaining % 10), COLOR_RED, COLOR_BLACK);
}

// Draw title screen
void draw_title_screen() {
    display_clear();
    
    // Animated starfield
    draw_stars();
    
    // Title
    text40_puts_color(12, 8, "== STAR RAIDERS ==", COLOR_YELLOW, COLOR_BLACK);
    text40_puts_color(11, 10, "A Space Shooter Game", COLOR_BLUE, COLOR_BLACK);
    
    // Instructions
    text40_puts_color(14, 13, "CONTROLS:", COLOR_GREEN, COLOR_BLACK);
    text40_puts_color(12, 14, "A/D - Move Left/Right", COLOR_WHITE, COLOR_BLACK);
    text40_puts_color(12, 15, "W/S - Move Up/Down", COLOR_WHITE, COLOR_BLACK);
    text40_puts_color(12, 16, "Space - Fire", COLOR_WHITE, COLOR_BLACK);
    
    // Power-ups legend
    text40_puts_color(14, 18, "POWER-UPS:", COLOR_PINK, COLOR_BLACK);
    text40_puts_color(10, 19, "R=Rapid S=Spread D=Shield L=Laser", COLOR_LIGHT_GRAY, COLOR_BLACK);
    
    // High score
    text40_puts_color(12, 21, "HIGH SCORE:", COLOR_ORANGE, COLOR_BLACK);
    for (int i = 5; i >= 0; i--) {
        int digit = high_score;
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(24 + (5 - i), 21, '0' + (digit % 10), COLOR_YELLOW, COLOR_BLACK);
    }
    
    // Start prompt
    if (frame_count % 30 < 20) {
        text40_puts_color(11, 23, "PRESS SPACE TO START", COLOR_WHITE, COLOR_BLACK);
    }
}

// Draw game over screen
void draw_gameover_screen() {
    display_clear();
    
    // Background effects
    for (int i = 0; i < 10; i++) {
        int x = rand_range(SCREEN_WIDTH);
        int y = rand_range(SCREEN_HEIGHT);
        text40_putchar_color(x, y, '*', COLOR_DARK_GRAY, COLOR_BLACK);
    }
    
    text40_puts_color(14, 10, "GAME OVER", COLOR_RED, COLOR_BLACK);
    
    text40_puts_color(13, 12, "FINAL SCORE:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 5; i >= 0; i--) {
        int digit = score;
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(17 + (5 - i), 13, '0' + (digit % 10), COLOR_YELLOW, COLOR_BLACK);
    }
    
    text40_puts_color(13, 15, "WAVES CLEARED:", COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(28, 15, '0' + (wave / 10), COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(29, 15, '0' + (wave % 10), COLOR_GREEN, COLOR_BLACK);
    
    if (score > high_score) {
        high_score = score;
        text40_puts_color(12, 17, "NEW HIGH SCORE!", COLOR_YELLOW, COLOR_RED);
    }
    
    if (frame_count % 30 < 20) {
        text40_puts_color(9, 20, "PRESS SPACE TO PLAY AGAIN", COLOR_WHITE, COLOR_BLACK);
    }
}

// Draw victory screen
void draw_victory_screen() {
    display_clear();
    
    // Fireworks effect
    for (int i = 0; i < 15; i++) {
        int x = rand_range(SCREEN_WIDTH);
        int y = rand_range(SCREEN_HEIGHT);
        unsigned char colors[4];
        colors[0] = COLOR_YELLOW;
        colors[1] = COLOR_PINK;
        colors[2] = COLOR_GREEN;
        colors[3] = COLOR_BLUE;
        text40_putchar_color(x, y, '*', colors[rand_range(4)], COLOR_BLACK);
    }
    
    text40_puts_color(14, 10, "VICTORY!", COLOR_YELLOW, COLOR_BLACK);
    text40_puts_color(10, 12, "You saved the galaxy!", COLOR_GREEN, COLOR_BLACK);
    
    text40_puts_color(13, 14, "FINAL SCORE:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 5; i >= 0; i--) {
        int digit = score;
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(17 + (5 - i), 15, '0' + (digit % 10), COLOR_YELLOW, COLOR_BLACK);
    }
    
    if (score > high_score) {
        high_score = score;
        text40_puts_color(12, 17, "NEW HIGH SCORE!", COLOR_YELLOW, COLOR_RED);
    }
    
    text40_puts_color(11, 20, "Thanks for playing!", COLOR_PINK, COLOR_BLACK);
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Get demo input
char get_demo_input() {
    // Find nearest enemy bullet to dodge
    int nearest_bullet_x = -1;
    int nearest_bullet_y = -1;
    int min_dist = 999;
    
    for (int i = 0; i < MAX_ENEMY_BULLETS; i++) {
        if (!enemy_bullet_active[i]) continue;
        
        int dist = (enemy_bullet_y[i] - player_y);
        if (dist > 0 && dist < min_dist && enemy_bullet_y[i] < player_y + 5) {
            min_dist = dist;
            nearest_bullet_x = enemy_bullet_x[i];
            nearest_bullet_y = enemy_bullet_y[i];
        }
    }
    
    // Dodge bullets
    if (nearest_bullet_x >= 0 && min_dist < 4) {
        if (nearest_bullet_x < player_x) {
            return 'd'; // Move right
        } else if (nearest_bullet_x > player_x) {
            return 'a'; // Move left
        }
    }
    
    // Find nearest enemy to shoot
    int target_x = -1;
    min_dist = 999;
    
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (!enemy_active[i]) continue;
        
        int dist = ((enemy_x[i] - player_x) * (enemy_x[i] - player_x)) +
                  ((enemy_y[i] - player_y) * (enemy_y[i] - player_y));
        
        if (dist < min_dist) {
            min_dist = dist;
            target_x = enemy_x[i];
        }
    }
    
    // Move towards enemy
    if (target_x >= 0) {
        if (target_x < player_x - 1) {
            return 'a';
        } else if (target_x > player_x + 1) {
            return 'd';
        }
    }
    
    // Always shoot
    return ' ';
}

int main() {
    // Enable TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Initialize game
    init_game();
    
    int shoot_cooldown = 0;
    int demo_duration = 3000; // Demo frame limit
    
    // Main game loop
    while (frame_count < demo_duration) {
        frame_count++;
        
        if (game_state == STATE_TITLE) {
            // Title screen
            draw_title_screen();
            update_stars();
            
            // Auto-start after a delay
            if (frame_count > 100) {
                game_state = STATE_PLAYING;
                init_game();
                game_state = STATE_PLAYING; // Reset after init_game
                spawn_wave(wave);
            }
            
        } else if (game_state == STATE_PLAYING) {
            // Get AI input
            char input = get_demo_input();
            
            // Process input
            if (input == 'a') {
                move_player(-1, 0);
            } else if (input == 'd') {
                move_player(1, 0);
            } else if (input == 'w') {
                move_player(0, -1);
            } else if (input == 's') {
                move_player(0, 1);
            } else if (input == ' ' && shoot_cooldown <= 0) {
                fire_bullet(0);
                if (player_power_type == POWERUP_RAPID) {
                    shoot_cooldown = 3;
                } else {
                    shoot_cooldown = 8;
                }
            }
            
            if (shoot_cooldown > 0) shoot_cooldown--;
            
            // Update game state
            update_stars();
            update_bullets();
            update_enemies();
            update_particles();
            update_powerups();
            
            // Update timers
            if (player_invulnerable > 0) player_invulnerable--;
            if (player_power_timer > 0) {
                player_power_timer--;
                if (player_power_timer == 0) {
                    player_power_type = -1;
                }
            }
            if (combo_timer > 0) {
                combo_timer--;
                if (combo_timer == 0) {
                    combo_counter = 0;
                }
            }
            
            // Check wave completion
            if (enemies_remaining == 0) {
                wave++;
                if (wave > 10) {
                    game_state = STATE_WIN;
                } else {
                    spawn_wave(wave);
                }
            }
            
            // Draw everything
            draw_game();
            
        } else if (game_state == STATE_GAMEOVER) {
            draw_gameover_screen();
            
            // Auto-restart after delay
            if (frame_count % 200 == 199) {
                init_game();
                game_state = STATE_PLAYING;
                spawn_wave(wave);
            }
            
        } else if (game_state == STATE_WIN) {
            draw_victory_screen();
            
            // Auto-restart after delay
            if (frame_count % 300 == 299) {
                init_game();
                game_state = STATE_PLAYING;
                spawn_wave(wave);
            }
        }
        
        display_flush();
        delay(2000); // Game speed
    }
    
    // End demo
    display_clear();
    text40_puts_color(12, 11, "DEMO COMPLETE", COLOR_WHITE, COLOR_BLUE);
    text40_puts_color(10, 13, "Thanks for watching!", COLOR_GREEN, COLOR_BLACK);
    
    text40_puts_color(12, 15, "HIGH SCORE:", COLOR_ORANGE, COLOR_BLACK);
    for (int i = 5; i >= 0; i--) {
        int digit = high_score;
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(24 + (5 - i), 15, '0' + (digit % 10), COLOR_YELLOW, COLOR_BLACK);
    }
    
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Star Raiders demo completed!");
    
    return 0;
}