// Automatic Snake Game Demo
// Features a self-playing snake that navigates toward food

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Leave 1 cell border on each side
#define GRID_WIDTH 38
// Leave 1 cell border on top/bottom  
#define GRID_HEIGHT 23
#define MAX_SNAKE_LENGTH 200
#define INITIAL_LENGTH 5
#define GAME_SPEED 5000

// Direction constants
#define DIR_UP    0
#define DIR_RIGHT 1
#define DIR_DOWN  2
#define DIR_LEFT  3

// Game objects
#define EMPTY  0
#define SNAKE  1
#define FOOD   2
#define WALL   3

// Snake structure
struct SnakeSegment {
    int x;
    int y;
};

struct SnakeSegment snake[MAX_SNAKE_LENGTH];
int snake_length;
int snake_direction;
int food_x;
int food_y;
int score;
int game_frame;

// Simple delay
void delay(int count) {
    for (int i = 0; i < count; i++) {
        int dummy = i * 2;
    }
}

// Get random number in range [0, max)
int rand_range(int max) {
    return rng_get() % max;
}

// Initialize the game
void init_game() {
    // Initialize snake in center
    snake_length = INITIAL_LENGTH;
    int start_x = GRID_WIDTH / 2;
    int start_y = GRID_HEIGHT / 2;
    
    for (int i = 0; i < INITIAL_LENGTH; i++) {
        snake[i].x = start_x - i;
        snake[i].y = start_y;
    }
    
    snake_direction = DIR_RIGHT;
    score = 0;
    game_frame = 0;
    
    // Place first food
    food_x = rand_range(GRID_WIDTH - 2) + 1;
    food_y = rand_range(GRID_HEIGHT - 2) + 1;
}

// Draw the game border
void draw_border() {
    // Top and bottom borders
    for (int x = 0; x < 40; x++) {
        text40_putchar_color(x, 0, '=', COLOR_BLUE, COLOR_DARK_BLUE);
        text40_putchar_color(x, 24, '=', COLOR_BLUE, COLOR_DARK_BLUE);
    }
    
    // Left and right borders
    for (int y = 1; y < 24; y++) {
        text40_putchar_color(0, y, '|', COLOR_BLUE, COLOR_DARK_BLUE);
        text40_putchar_color(39, y, '|', COLOR_BLUE, COLOR_DARK_BLUE);
    }
    
    // Corners
    text40_putchar_color(0, 0, '+', COLOR_YELLOW, COLOR_DARK_BLUE);
    text40_putchar_color(39, 0, '+', COLOR_YELLOW, COLOR_DARK_BLUE);
    text40_putchar_color(0, 24, '+', COLOR_YELLOW, COLOR_DARK_BLUE);
    text40_putchar_color(39, 24, '+', COLOR_YELLOW, COLOR_DARK_BLUE);
    
    // Title
    text40_puts_color(14, 0, " SNAKE DEMO ", COLOR_YELLOW, COLOR_DARK_BLUE);
    
    // Score display
    text40_puts_color(2, 24, "Score:", COLOR_WHITE, COLOR_DARK_BLUE);
}

// Draw the snake
void draw_snake() {
    // Draw head with special color
    text40_putchar_color(snake[0].x + 1, snake[0].y + 1, 'O', 
                        COLOR_YELLOW, COLOR_DARK_GREEN);
    
    // Draw body segments
    for (int i = 1; i < snake_length; i++) {
        char body_char = (i == snake_length - 1) ? 'o' : '*';
        unsigned char color = (i % 2 == 0) ? COLOR_GREEN : COLOR_DARK_GREEN;
        text40_putchar_color(snake[i].x + 1, snake[i].y + 1, body_char, 
                           COLOR_WHITE, color);
    }
}

// Draw the food
void draw_food() {
    // Pulsing food animation
    unsigned char food_color = ((game_frame / 4) % 2 == 0) ? COLOR_RED : COLOR_ORANGE;
    text40_putchar_color(food_x + 1, food_y + 1, '@', COLOR_WHITE, food_color);
}

// Clear a position on the grid
void clear_position(int x, int y) {
    text40_putchar_color(x + 1, y + 1, ' ', COLOR_BLACK, COLOR_BLACK);
}

// Update score display
void update_score() {
    // Display score as 3 digits
    int hundreds = score / 100;
    int tens = (score / 10) % 10;
    int ones = score % 10;
    
    text40_putchar_color(9, 24, '0' + hundreds, COLOR_WHITE, COLOR_DARK_BLUE);
    text40_putchar_color(10, 24, '0' + tens, COLOR_WHITE, COLOR_DARK_BLUE);
    text40_putchar_color(11, 24, '0' + ones, COLOR_WHITE, COLOR_DARK_BLUE);
}

// Simple AI to determine next direction
int get_ai_direction() {
    int head_x = snake[0].x;
    int head_y = snake[0].y;
    int dx = food_x - head_x;
    int dy = food_y - head_y;
    int can_go_up = 1;
    int can_go_down = 1;
    int can_go_left = 1;
    int can_go_right = 1;
    int check_limit;
    int i;
    
    // Check boundaries
    if (head_y <= 0) can_go_up = 0;
    if (head_y >= GRID_HEIGHT - 1) can_go_down = 0;
    if (head_x <= 0) can_go_left = 0;
    if (head_x >= GRID_WIDTH - 1) can_go_right = 0;
    
    // Check for self-collision (only check first few segments for simplicity)
    check_limit = (snake_length < 10) ? snake_length : 10;
    for (int i = 1; i < check_limit; i++) {
        if (snake[i].x == head_x && snake[i].y == head_y - 1) can_go_up = 0;
        if (snake[i].x == head_x && snake[i].y == head_y + 1) can_go_down = 0;
        if (snake[i].x == head_x - 1 && snake[i].y == head_y) can_go_left = 0;
        if (snake[i].x == head_x + 1 && snake[i].y == head_y) can_go_right = 0;
    }
    
    // Don't reverse direction
    if (snake_direction == DIR_UP) can_go_down = 0;
    if (snake_direction == DIR_DOWN) can_go_up = 0;
    if (snake_direction == DIR_LEFT) can_go_right = 0;
    if (snake_direction == DIR_RIGHT) can_go_left = 0;
    
    // Simple AI: Try to move toward food
    // Prioritize the axis with greater distance
    if (dx > 0 && can_go_right) {
        return DIR_RIGHT;
    } else if (dx < 0 && can_go_left) {
        return DIR_LEFT;
    } else if (dy > 0 && can_go_down) {
        return DIR_DOWN;
    } else if (dy < 0 && can_go_up) {
        return DIR_UP;
    }
    
    // If we can't move toward food, try any valid direction
    if (can_go_right && snake_direction != DIR_LEFT) return DIR_RIGHT;
    if (can_go_down && snake_direction != DIR_UP) return DIR_DOWN;
    if (can_go_left && snake_direction != DIR_RIGHT) return DIR_LEFT;
    if (can_go_up && snake_direction != DIR_DOWN) return DIR_UP;
    
    // If no valid move, keep current direction (will crash)
    return snake_direction;
}

// Move the snake
int move_snake() {
    int new_x;
    int new_y;
    int ate_food;
    int i;
    int placed;
    int on_snake;
    
    // Get AI decision
    snake_direction = get_ai_direction();
    
    // Calculate new head position
    new_x = snake[0].x;
    new_y = snake[0].y;
    
    if (snake_direction == DIR_UP) new_y--;
    else if (snake_direction == DIR_DOWN) new_y++;
    else if (snake_direction == DIR_LEFT) new_x--;
    else if (snake_direction == DIR_RIGHT) new_x++;
    
    // Check collision with walls
    if (new_x < 0 || new_x >= GRID_WIDTH || 
        new_y < 0 || new_y >= GRID_HEIGHT) {
        return 0; // Game over
    }
    
    // Check collision with self
    for (i = 0; i < snake_length; i++) {
        if (snake[i].x == new_x && snake[i].y == new_y) {
            return 0; // Game over
        }
    }
    
    // Check if food is eaten
    ate_food = (new_x == food_x && new_y == food_y);
    
    // Clear tail if not growing
    if (!ate_food) {
        clear_position(snake[snake_length - 1].x, snake[snake_length - 1].y);
    }
    
    // Move body segments
    for (i = snake_length - 1; i > 0; i--) {
        snake[i].x = snake[i - 1].x;
        snake[i].y = snake[i - 1].y;
    }
    
    // Move head
    snake[0].x = new_x;
    snake[0].y = new_y;
    
    // Handle food consumption
    if (ate_food) {
        score += 10;
        if (snake_length < MAX_SNAKE_LENGTH) {
            snake_length++;
        }
        
        // Place new food
        placed = 0;
        while (!placed) {
            food_x = rand_range(GRID_WIDTH - 2) + 1;
            food_y = rand_range(GRID_HEIGHT - 2) + 1;
            
            // Make sure food doesn't spawn on snake
            on_snake = 0;
            for (i = 0; i < snake_length; i++) {
                if (snake[i].x == food_x && snake[i].y == food_y) {
                    on_snake = 1;
                    i = snake_length; // Exit loop
                }
            }
            if (!on_snake) placed = 1;
        }
    }
    
    return 1; // Continue game
}

// Draw special effects when eating food
void draw_eat_effect() {
    // Flash effect around food position
    for (int dx = -1; dx <= 1; dx++) {
        for (int dy = -1; dy <= 1; dy++) {
            if (dx == 0 && dy == 0) continue;
            int x = food_x + dx + 1;
            int y = food_y + dy + 1;
            if (x > 0 && x < 39 && y > 0 && y < 24) {
                text40_putchar_color(x, y, '+', COLOR_YELLOW, COLOR_BLACK);
            }
        }
    }
}

// Game over screen
void game_over() {
    // Draw game over message
    text40_puts_color(14, 10, " GAME OVER! ", COLOR_WHITE, COLOR_RED);
    text40_puts_color(13, 12, " Final Score: ", COLOR_YELLOW, COLOR_DARK_BLUE);
    
    // Display final score
    text40_putchar_color(27, 12, '0' + (score / 100), COLOR_WHITE, COLOR_DARK_BLUE);
    text40_putchar_color(28, 12, '0' + ((score / 10) % 10), COLOR_WHITE, COLOR_DARK_BLUE);
    text40_putchar_color(29, 12, '0' + (score % 10), COLOR_WHITE, COLOR_DARK_BLUE);
    
    // Show length
    text40_puts_color(12, 14, " Snake Length: ", COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(27, 14, '0' + (snake_length / 10), COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(28, 14, '0' + (snake_length % 10), COLOR_WHITE, COLOR_BLACK);
}

int main() {
    // Enable TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Set random seed based on time (using RNG)
    rng_set_seed(rng_get() + 42);
    
    // Initialize game
    init_game();
    
    // Draw static elements
    draw_border();
    
    // Main game loop
    int game_running = 1;
    int frames_since_food = 0;
    
    for (int frame = 0; frame < 500 && game_running; frame++) {
        game_frame = frame;
        
        // Move snake every few frames
        if (frame % 3 == 0) {
            game_running = move_snake();
            
            // Check if food was just eaten
            if (score > 0 && score % 10 == 0 && frames_since_food < 3) {
                draw_eat_effect();
                frames_since_food++;
            } else {
                frames_since_food = 0;
            }
        }
        
        // Draw game elements
        draw_snake();
        draw_food();
        update_score();
        
        // Add some visual flair - animated border
        if (frame % 10 == 0) {
            unsigned char border_color = ((frame / 10) % 2 == 0) ? COLOR_BLUE : COLOR_DARK_BLUE;
            for (int i = 5; i < 35; i += 5) {
                text40_putchar_color(i, 0, '=', COLOR_WHITE, border_color);
                text40_putchar_color(i, 24, '=', COLOR_WHITE, border_color);
            }
        }
        
        display_flush();
        delay(GAME_SPEED);
    }
    
    // Show game over screen
    if (!game_running) {
        game_over();
        display_flush();
        delay(800);
    }
    
    // Return to normal mode
    display_set_mode(DISP_MODE_OFF);
    puts("Snake demo complete!");
    
    return 0;
}