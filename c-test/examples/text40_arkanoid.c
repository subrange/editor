// Arkanoid (Breakout) game for TEXT40 display
// Features paddle control, ball physics, brick breaking, and power-ups

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Game dimensions
#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25
#define PADDLE_WIDTH 7
#define PADDLE_Y 22
#define BRICKS_START_Y 3
#define BRICKS_ROWS 6
#define BRICKS_COLS 10
#define BRICK_WIDTH 3
#define MAX_BALLS 3
#define MAX_POWERUPS 5

// Game state
int paddle_x;
int paddle_width;
int score;
int lives;
int level;
int game_over;
int game_won;
int bricks_remaining;

// Ball structure
struct Ball {
    int x;      // Fixed point (actual * 256)
    int y;      // Fixed point (actual * 256)
    int vx;     // Velocity X
    int vy;     // Velocity Y
    int active;
};

// Power-up types
#define POWERUP_NONE 0
#define POWERUP_EXPAND 1
#define POWERUP_MULTI 2
#define POWERUP_SLOW 3
#define POWERUP_LIFE 4
#define POWERUP_POWER 5

// Power-up structure
struct PowerUp {
    int x;
    int y;      // Fixed point
    int type;
    int active;
};

// Brick structure (using array since 2D arrays don't work)
unsigned char bricks[60];  // BRICKS_ROWS * BRICKS_COLS
unsigned char brick_hits[60];  // Hit points for each brick

struct Ball balls[MAX_BALLS];
struct PowerUp powerups[MAX_POWERUPS];
int power_ball_timer;
int multi_ball_active;

// Get random number in range
int rand_range(int max) {
    return (rng_get() % max);
}

// Get brick index
int brick_index(int row, int col) {
    return row * BRICKS_COLS + col;
}

// Initialize level
void init_level(int level_num) {
    // Clear bricks
    bricks_remaining = 0;
    
    // Create brick pattern based on level
    for (int row = 0; row < BRICKS_ROWS; row++) {
        for (int col = 0; col < BRICKS_COLS; col++) {
            int idx = brick_index(row, col);
            
            // Different patterns for different levels
            if (level_num == 1) {
                // Simple pattern - all bricks
                bricks[idx] = 1;
                brick_hits[idx] = 1;
            } else if (level_num == 2) {
                // Checkerboard pattern
                bricks[idx] = ((row + col) % 2 == 0) ? 1 : 0;
                brick_hits[idx] = 1;
            } else if (level_num == 3) {
                // Diamond pattern
                int center_col = BRICKS_COLS / 2;
                int center_row = BRICKS_ROWS / 2;
                int dist = ((row - center_row) * (row - center_row)) + 
                          ((col - center_col) * (col - center_col));
                bricks[idx] = (dist < 12) ? 1 : 0;
                brick_hits[idx] = (row < 2) ? 2 : 1;  // Top rows need 2 hits
            } else {
                // Random pattern with varying hit points
                bricks[idx] = (rand_range(100) < 70) ? 1 : 0;
                brick_hits[idx] = 1 + (row < 2 ? 1 : 0);
            }
            
            if (bricks[idx]) {
                bricks_remaining++;
            }
        }
    }
}

// Initialize game
void init_game() {
    paddle_x = SCREEN_WIDTH / 2 - PADDLE_WIDTH / 2;
    paddle_width = PADDLE_WIDTH;
    score = 0;
    lives = 3;
    level = 1;
    game_over = 0;
    game_won = 0;
    power_ball_timer = 0;
    multi_ball_active = 0;
    
    // Initialize first ball
    balls[0].x = (SCREEN_WIDTH / 2) * 256;
    balls[0].y = 20 * 256;
    balls[0].vx = 256 + rand_range(128) - 64;  // Random angle
    balls[0].vy = -256;
    balls[0].active = 1;
    
    // Deactivate other balls
    for (int i = 1; i < MAX_BALLS; i++) {
        balls[i].active = 0;
    }
    
    // Clear power-ups
    for (int i = 0; i < MAX_POWERUPS; i++) {
        powerups[i].active = 0;
    }
    
    init_level(level);
}

// Move paddle
void move_paddle(int direction) {
    paddle_x += direction * 2;
    if (paddle_x < 1) paddle_x = 1;
    if (paddle_x + paddle_width > SCREEN_WIDTH - 1) {
        paddle_x = SCREEN_WIDTH - 1 - paddle_width;
    }
}

// Spawn power-up
void spawn_powerup(int x, int y) {
    // 30% chance to spawn power-up
    if (rand_range(100) < 30) {
        for (int i = 0; i < MAX_POWERUPS; i++) {
            if (!powerups[i].active) {
                powerups[i].x = x;
                powerups[i].y = y * 256;
                powerups[i].type = 1 + rand_range(5);  // Random power-up type
                powerups[i].active = 1;
                break;
            }
        }
    }
}

// Check brick collision
void check_brick_collision(struct Ball* ball) {
    int ball_x = ball->x / 256;
    int ball_y = ball->y / 256;
    
    // Check which brick zone we're in
    int brick_y = ball_y - BRICKS_START_Y;
    if (brick_y >= 0 && brick_y < BRICKS_ROWS * 2) {
        int row = brick_y / 2;
        int col = (ball_x - 2) / BRICK_WIDTH;
        
        if (col >= 0 && col < BRICKS_COLS && row >= 0 && row < BRICKS_ROWS) {
            int idx = brick_index(row, col);
            
            if (bricks[idx] > 0) {
                // Hit brick
                if (power_ball_timer > 0) {
                    // Power ball destroys in one hit
                    brick_hits[idx] = 0;
                } else {
                    brick_hits[idx]--;
                }
                
                if (brick_hits[idx] <= 0) {
                    bricks[idx] = 0;
                    bricks_remaining--;
                    score += 10 * level;
                    
                    // Spawn power-up at brick location
                    spawn_powerup(2 + col * BRICK_WIDTH + 1, 
                                 BRICKS_START_Y + row * 2);
                } else {
                    score += 5;  // Points for damaging brick
                }
                
                // Bounce ball
                ball->vy = -ball->vy;
                
                // Add some randomness to prevent infinite loops
                ball->vx += rand_range(64) - 32;
            }
        }
    }
}

// Update ball physics
void update_ball(struct Ball* ball) {
    if (!ball->active) return;
    
    // Update position
    ball->x += ball->vx;
    ball->y += ball->vy;
    
    int ball_x = ball->x / 256;
    int ball_y = ball->y / 256;
    
    // Wall collisions
    if (ball_x <= 0 || ball_x >= SCREEN_WIDTH - 1) {
        ball->vx = -ball->vx;
        ball->x = (ball_x <= 0) ? 256 : (SCREEN_WIDTH - 2) * 256;
    }
    
    // Ceiling collision
    if (ball_y <= 1) {
        ball->vy = -ball->vy;
        ball->y = 2 * 256;
    }
    
    // Paddle collision
    if (ball_y >= PADDLE_Y - 1 && ball_y <= PADDLE_Y && 
        ball_x >= paddle_x && ball_x < paddle_x + paddle_width) {
        
        ball->vy = -ball->vy;
        ball->y = (PADDLE_Y - 1) * 256;
        
        // Adjust angle based on where ball hits paddle
        int paddle_hit = ball_x - paddle_x;
        ball->vx = -256 + (paddle_hit * 512 / paddle_width);
        
        // Ensure minimum horizontal velocity
        if (ball->vx > -64 && ball->vx < 64) {
            ball->vx = (ball->vx < 0) ? -64 : 64;
        }
    }
    
    // Ball lost
    if (ball_y >= SCREEN_HEIGHT) {
        ball->active = 0;
        
        // Check if any balls remain
        int balls_left = 0;
        for (int i = 0; i < MAX_BALLS; i++) {
            if (balls[i].active) balls_left++;
        }
        
        if (balls_left == 0) {
            lives--;
            if (lives > 0) {
                // Reset ball
                balls[0].x = (SCREEN_WIDTH / 2) * 256;
                balls[0].y = 20 * 256;
                balls[0].vx = 256;
                balls[0].vy = -256;
                balls[0].active = 1;
                paddle_width = PADDLE_WIDTH;  // Reset paddle size
            } else {
                game_over = 1;
            }
        }
    }
    
    // Check brick collisions
    check_brick_collision(ball);
    
    // Check for level complete
    if (bricks_remaining == 0) {
        level++;
        if (level > 4) {
            game_won = 1;
        } else {
            init_level(level);
            // Reset ball for new level
            balls[0].x = (SCREEN_WIDTH / 2) * 256;
            balls[0].y = 20 * 256;
            balls[0].vx = 256;
            balls[0].vy = -256;
            balls[0].active = 1;
            for (int i = 1; i < MAX_BALLS; i++) {
                balls[i].active = 0;
            }
        }
    }
}

// Update power-ups
void update_powerups() {
    for (int i = 0; i < MAX_POWERUPS; i++) {
        if (!powerups[i].active) continue;
        
        // Fall down
        powerups[i].y += 128;
        int py = powerups[i].y / 256;
        
        // Check paddle collision
        if (py >= PADDLE_Y && py <= PADDLE_Y + 1 &&
            powerups[i].x >= paddle_x && powerups[i].x < paddle_x + paddle_width) {
            
            // Apply power-up effect
            if (powerups[i].type == POWERUP_EXPAND) {
                paddle_width = PADDLE_WIDTH + 3;
                if (paddle_width > 12) paddle_width = 12;
                score += 50;
            } else if (powerups[i].type == POWERUP_MULTI) {
                // Spawn extra balls
                for (int j = 1; j < MAX_BALLS; j++) {
                    if (!balls[j].active) {
                        balls[j].x = balls[0].x;
                        balls[j].y = balls[0].y;
                        balls[j].vx = balls[0].vx + rand_range(256) - 128;
                        balls[j].vy = -256;
                        balls[j].active = 1;
                        multi_ball_active = 1;
                    }
                }
                score += 100;
            } else if (powerups[i].type == POWERUP_SLOW) {
                // Slow down balls
                for (int j = 0; j < MAX_BALLS; j++) {
                    if (balls[j].active) {
                        balls[j].vx = balls[j].vx / 2;
                        balls[j].vy = balls[j].vy / 2;
                    }
                }
                score += 30;
            } else if (powerups[i].type == POWERUP_LIFE) {
                lives++;
                score += 200;
            } else if (powerups[i].type == POWERUP_POWER) {
                power_ball_timer = 300;  // Power ball for 300 frames
                score += 75;
            }
            
            powerups[i].active = 0;
        }
        
        // Remove if off screen
        if (py >= SCREEN_HEIGHT) {
            powerups[i].active = 0;
        }
    }
}

// Draw game
void draw_game() {
    // Clear screen
    display_clear();
    
    // Draw border
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, 0, '=', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(x, SCREEN_HEIGHT - 1, '=', COLOR_BLUE, COLOR_BLACK);
    }
    for (int y = 1; y < SCREEN_HEIGHT - 1; y++) {
        text40_putchar_color(0, y, '|', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(SCREEN_WIDTH - 1, y, '|', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Draw bricks
    for (int row = 0; row < BRICKS_ROWS; row++) {
        for (int col = 0; col < BRICKS_COLS; col++) {
            int idx = brick_index(row, col);
            if (bricks[idx] > 0) {
                int x = 2 + col * BRICK_WIDTH;
                int y = BRICKS_START_Y + row * 2;
                
                // Color based on row and hits
                unsigned char color;
                if (brick_hits[idx] > 1) {
                    color = COLOR_RED;  // Strong brick
                } else if (row < 2) {
                    color = COLOR_ORANGE;
                } else if (row < 4) {
                    color = COLOR_YELLOW;
                } else {
                    color = COLOR_GREEN;
                }
                
                // Draw brick
                for (int bx = 0; bx < BRICK_WIDTH - 1; bx++) {
                    text40_putchar_color(x + bx, y, '=', color, COLOR_BLACK);
                }
            }
        }
    }
    
    // Draw paddle
    for (int i = 0; i < paddle_width; i++) {
        text40_putchar_color(paddle_x + i, PADDLE_Y, '=', COLOR_WHITE, COLOR_BLACK);
    }
    
    // Draw balls
    for (int i = 0; i < MAX_BALLS; i++) {
        if (balls[i].active) {
            int bx = balls[i].x / 256;
            int by = balls[i].y / 256;
            if (bx >= 0 && bx < SCREEN_WIDTH && by >= 0 && by < SCREEN_HEIGHT) {
                unsigned char ball_color = power_ball_timer > 0 ? COLOR_YELLOW : COLOR_WHITE;
                text40_putchar_color(bx, by, 'o', ball_color, COLOR_BLACK);
            }
        }
    }
    
    // Draw power-ups
    for (int i = 0; i < MAX_POWERUPS; i++) {
        if (powerups[i].active) {
            int py = powerups[i].y / 256;
            if (py >= 0 && py < SCREEN_HEIGHT) {
                char powerup_char;
                unsigned char powerup_color;
                
                if (powerups[i].type == POWERUP_EXPAND) {
                    powerup_char = 'E';
                    powerup_color = COLOR_GREEN;
                } else if (powerups[i].type == POWERUP_MULTI) {
                    powerup_char = 'M';
                    powerup_color = COLOR_BLUE;
                } else if (powerups[i].type == POWERUP_SLOW) {
                    powerup_char = 'S';
                    powerup_color = COLOR_INDIGO;
                } else if (powerups[i].type == POWERUP_LIFE) {
                    powerup_char = 'L';
                    powerup_color = COLOR_PINK;
                } else if (powerups[i].type == POWERUP_POWER) {
                    powerup_char = 'P';
                    powerup_color = COLOR_YELLOW;
                } else {
                    powerup_char = '?';
                    powerup_color = COLOR_WHITE;
                }
                
                text40_putchar_color(powerups[i].x, py, powerup_char, 
                                   powerup_color, COLOR_BLACK);
            }
        }
    }
    
    // Draw UI
    text40_puts_color(1, 0, " ARKANOID ", COLOR_WHITE, COLOR_DARK_BLUE);
    
    // Score
    text40_puts_color(12, 0, "SCORE:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 5; i >= 0; i--) {
        int digit = (score / 1);
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(19 - i, 0, '0' + (digit % 10), COLOR_YELLOW, COLOR_BLACK);
    }
    
    // Lives
    text40_puts_color(26, 0, "LIVES:", COLOR_WHITE, COLOR_BLACK);
    for (int i = 0; i < lives && i < 5; i++) {
        text40_putchar_color(33 + i, 0, '*', COLOR_RED, COLOR_BLACK);
    }
    
    // Level indicator
    text40_puts_color(1, 1, "LVL", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(5, 1, '0' + level, COLOR_GREEN, COLOR_BLACK);
    
    // Power-up indicator
    if (power_ball_timer > 0) {
        text40_puts_color(8, 1, "POWER!", COLOR_YELLOW, COLOR_RED);
    }
    
    // Bricks remaining
    text40_puts_color(30, 1, "LEFT:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(36, 1, '0' + (bricks_remaining / 10), COLOR_ORANGE, COLOR_BLACK);
    text40_putchar_color(37, 1, '0' + (bricks_remaining % 10), COLOR_ORANGE, COLOR_BLACK);
}

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Get demo input (simulated)
char get_demo_input(int frame, int ball_x) {
    // Simple AI: try to follow the ball
    int target = ball_x - paddle_width / 2;
    
    if (paddle_x < target - 2) {
        return 'd';  // Move right
    } else if (paddle_x > target + 2) {
        return 'a';  // Move left
    }
    
    // Occasionally don't move for more realistic play
    if (rand_range(100) < 20) {
        return 0;
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
    int frame = 0;
    int max_frames = 2000;  // Demo duration
    
    while (!game_over && !game_won && frame < max_frames) {
        // Get input (demo AI)
        char input = get_demo_input(frame, balls[0].x / 256);
        
        if (input == 'a') {
            move_paddle(-1);
        } else if (input == 'd') {
            move_paddle(1);
        }
        
        // Update game physics
        for (int i = 0; i < MAX_BALLS; i++) {
            update_ball(&balls[i]);
        }
        
        update_powerups();
        
        // Update timers
        if (power_ball_timer > 0) {
            power_ball_timer--;
        }
        
        // Draw everything
        draw_game();
        display_flush();
        
        // Frame delay
        delay(3000);
        frame++;
    }
    
    // End screen
    if (game_won) {
        text40_puts_color(13, 11, " YOU WIN! ", COLOR_BLACK, COLOR_YELLOW);
        text40_puts_color(10, 13, " FINAL SCORE ", COLOR_WHITE, COLOR_GREEN);
    } else if (game_over) {
        text40_puts_color(12, 11, " GAME OVER ", COLOR_WHITE, COLOR_RED);
        text40_puts_color(10, 13, " FINAL SCORE ", COLOR_BLACK, COLOR_YELLOW);
    } else {
        text40_puts_color(11, 11, " DEMO COMPLETE ", COLOR_WHITE, COLOR_BLUE);
    }
    
    // Display final score
    for (int i = 5; i >= 0; i--) {
        int digit = (score / 1);
        for (int j = 0; j < i; j++) digit /= 10;
        text40_putchar_color(17 - i, 14, '0' + (digit % 10), COLOR_YELLOW, COLOR_BLACK);
    }
    
    display_flush();
    delay(50000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Arkanoid demo completed!");
    
    return 0;
}