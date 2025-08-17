// Doom-like raycasted 3D FPS for TEXT40 display
// Uses fixed-point math (16-bit integers) and 1D arrays

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

#define PLAYER_R 3   // collision radius in subunits (0..15) ≈ 0.19 tile

#define S_MOVE_FORWARD 0
#define S_ROTATE_UNTIL_FREE 1

int player_state = S_MOVE_FORWARD;

int wall_cell(int tx, int ty){ return get_map(tx,ty) != 0; }

// Display and rendering constants
#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25
#define RENDER_HEIGHT 20  // Leave room for HUD
#define FOV 60            // Field of view in degrees
#define NUM_RAYS 40       // One ray per column

#define FOV256 ((FOV * 256) / 360)

// Fixed-point math (Q12.4)
#define FP_SHIFT 4
#define FP_SCALE 16
#define FP_HALF  (FP_SCALE/2)   // 8
#define FP_ONE   (FP_SCALE)     // 16

// Map dimensions
#define MAP_WIDTH 16
#define MAP_HEIGHT 16
#define MAP_SIZE 256  // 16x16

// Maximum ray distance
#define MAX_DIST (16 * FP_SCALE)   // 256
#define WALL_HEIGHT 256

// Player movement
#define MOVE_SPEED (FP_SCALE + FP_SCALE/2)   // 24 in Q12.4
#define TURN_SPEED 5

// Texture size
#define TEX_WIDTH 8
#define TEX_HEIGHT 8
#define TEX_SIZE 64



// Game state
int player_x;      // Fixed point position
int player_y;
int player_angle;  // Angle in units of 256 = 360 degrees
int player_health;
int player_ammo;
int frame_count;

// Enemy state
int enemy_x;
int enemy_y;
int enemy_active;
int enemy_health;
int enemy_angle;

// Map data (1D array) - 1=wall, 0=empty, 2=door, 3=key
//unsigned char map_data[256] = {
//    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
//    1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1,
//    1,0,1,1,0,1,0,1,0,1,1,1,1,1,0,1,
//    1,0,1,0,0,1,0,0,0,0,0,0,0,1,0,1,
//    1,0,1,0,1,1,1,2,1,1,1,1,0,1,0,1,
//    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
//    1,1,1,0,1,0,1,1,1,1,0,1,0,1,1,1,
//    1,0,0,0,1,0,0,1,1,0,0,1,0,0,0,1,
//    1,0,1,1,1,1,0,1,1,0,1,1,1,1,0,1,
//    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
//    1,1,1,1,0,1,1,0,0,1,1,0,1,1,1,1,
//    1,0,0,0,0,0,1,0,0,1,0,0,0,0,3,1,
//    1,0,1,1,1,0,1,1,1,1,0,1,1,1,0,1,
//    1,0,0,0,1,0,0,0,0,0,0,1,0,0,0,1,
//    1,0,1,0,0,0,1,0,0,1,0,0,0,1,0,1,
//    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1
//};
//
// Empty map for testing
unsigned char map_data[MAP_SIZE] = {
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1

};

// Wall texture patterns (8x8, flattened)
unsigned char tex_brick[64] = {
    1,1,1,1,1,1,1,0,
    1,0,0,0,0,0,0,0,
    1,0,0,0,0,0,0,0,
    1,1,1,1,1,1,1,0,
    0,0,0,1,0,0,0,0,
    0,0,0,1,0,0,0,0,
    1,1,1,1,1,1,1,0,
    1,0,0,0,0,0,0,0
};

unsigned char tex_stone[64] = {
    1,0,1,1,1,0,1,1,
    0,1,0,1,0,1,0,1,
    1,1,0,0,0,1,1,0,
    1,0,1,0,1,0,1,0,
    0,1,1,1,1,1,0,1,
    1,0,0,1,0,0,1,1,
    1,1,0,1,0,1,1,0,
    0,1,1,0,1,1,0,1
};

unsigned char tex_tech[64] = {
    1,1,0,1,1,0,1,1,
    1,0,0,0,0,0,0,1,
    0,0,1,0,0,1,0,0,
    1,0,0,0,0,0,0,1,
    1,0,0,1,1,0,0,1,
    0,0,1,0,0,1,0,0,
    1,0,0,0,0,0,0,1,
    1,1,0,1,1,0,1,1
};

// Enemy sprite (8x8) - demon face
unsigned char sprite_enemy[64] = {
    0,0,1,1,1,1,0,0,
    0,1,0,1,1,0,1,0,
    1,1,0,0,0,0,1,1,
    1,0,1,0,0,1,0,1,
    1,0,0,1,1,0,0,1,
    1,1,0,1,1,0,1,1,
    0,1,1,0,0,1,1,0,
    0,0,1,1,1,1,0,0
};

// Weapon sprite (simple pistol at bottom)
unsigned char sprite_weapon[40] = {
    0,0,0,0,1,1,1,1,
    0,0,0,1,1,0,1,1,
    0,0,1,1,1,1,1,0,
    0,1,1,1,1,1,0,0,
    1,1,1,1,1,0,0,0
};

// Depth buffer for sprite rendering
int depth_buffer[40];

// Random number generation
int rand_range(int max) {
    return (rng_get() % max);
}

// Get sine from table
// was: / 16
//int cos_lookup(int angle) { return (sin_table[(angle + 64) & 255] + 8) >> 4; }
//int sin_lookup(int angle) { return (sin_table[angle & 255] + 8) >> 4; }

int sin_lookup(int a) {
    int q = a & 255;
    int sign = (q & 128) ? -1 : +1;
    int t = q & 63;                 // 0..63 within quadrant
    int ramp = (q & 64) ? (63 - t) : t; // up then down
    return sign * ((ramp * 16) / 63);   // 0..16 scaled
}
int cos_lookup(int a) { return sin_lookup((a + 64) & 255); }

int fp_mul(int a, int b) { return (a * b) >> FP_SHIFT; }

int fp_div(int a, int b) {
    if (b == 0) return MAX_DIST;
    return (a << FP_SHIFT) / b;
}

// Absolute value
int abs_val(int x) {
    return x < 0 ? -x : x;
}

// Get map tile at position
int get_map(int x, int y) {
    if (x < 0 || x >= MAP_WIDTH || y < 0 || y >= MAP_HEIGHT) {
        return 1;  // Outside bounds is wall
    }
    return map_data[y * MAP_WIDTH + x];
}

// Global variables to return multiple values from cast_ray
int last_hit_type;
int last_wall_x;
int last_hit_side;

// returns perpendicular (fisheye-free) distance in FP units
// sets: last_hit_type, last_hit_side (0=vertical,1=horizontal), last_wall_x (0..255)
int cast_ray(int ray_x, int ray_y, int ray_angle) {
    // ray direction in 8.8 (±256)
    int dirX = cos_lookup(ray_angle);
    int dirY = sin_lookup(ray_angle);

    // current map cell
    int mapX = ray_x >> FP_SHIFT;
    int mapY = ray_y >> FP_SHIFT;

    // step for x/y and initial side distances
    int stepX = (dirX >= 0) ? 1 : -1;
    int stepY = (dirY >= 0) ? 1 : -1;
    int absDirX = (dirX >= 0) ? dirX : -dirX;
    int absDirY = (dirY >= 0) ? dirY : -dirY;

    // distance (in ray "t" units, same FP as your 'dist') to next grid boundary
    // t such that fp_mul(t, |dir|) moves exactly one tile (256 units) along that axis
    // deltaDist = (256<<8) / |dir|
    int ONE_TILE_T = (FP_SCALE << FP_SHIFT);   // 16<<4 = 256 (fits 16-bit)
    int INF = 30000;                           // big number that still fits

    int deltaDistX = absDirX ? (ONE_TILE_T / absDirX) : INF;
    int deltaDistY = absDirY ? (ONE_TILE_T / absDirY) : INF;

    // distance from ray origin to first boundary on each axis
    int sideDistX, sideDistY;
    if (stepX > 0)
        sideDistX = fp_div(((mapX + 1) << FP_SHIFT) - ray_x, absDirX);
    else
        sideDistX = fp_div(ray_x - (mapX << FP_SHIFT), absDirX);

    if (stepY > 0)
        sideDistY = fp_div(((mapY + 1) << FP_SHIFT) - ray_y, absDirY);
    else
        sideDistY = fp_div(ray_y - (mapY << FP_SHIFT), absDirY);

    // DDA
    int side = -1; // 0 = vertical wall, 1 = horizontal wall
    int hit  = 0;
    for (int it = 0; it < 128; it++) {
        if (sideDistX < sideDistY) {
            sideDistX += deltaDistX;
            mapX += stepX;
            side = 0;
        } else {
            sideDistY += deltaDistY;
            mapY += stepY;
            side = 1;
        }

        int tile = get_map(mapX, mapY);
        if (tile > 0) {
            last_hit_type = tile;
            last_hit_side = side;

            // distance to wall along the ray (perpendicular distance)
            int dist = (side == 0 ? (sideDistX - deltaDistX) : (sideDistY - deltaDistY));

            // exact hit point for texture coordinate
            int hitX = ray_x + fp_mul(dist, dirX);
            int hitY = ray_y + fp_mul(dist, dirY);
            last_wall_x = (side == 0 ? hitY : hitX) & (FP_SCALE - 1); // 0..255

            if (dist <= 0) dist = 1;       // avoid div-by-zero up close
            if (dist > MAX_DIST) dist = MAX_DIST;
            return dist;
        }
    }

    // no hit within bounds
    last_hit_type = 0;
    last_hit_side = 0;
    last_wall_x   = 0;
    return MAX_DIST;
}
// Render the 3D view
void render_3d() {
    // Clear depth buffer
    for (int i = 0; i < NUM_RAYS; i++) {
        depth_buffer[i] = MAX_DIST;
    }
    
    // Cast rays for each column
    for (int col = 0; col < NUM_RAYS; col++) {
    // offset in 0..255 units, spanning [-FOV/2, +FOV/2]
    int ray_off = (((col * 2 + 1) * FOV256) / NUM_RAYS) - FOV256;
    int ray_angle = (player_angle + ray_off) & 255;

    int dist = cast_ray(player_x, player_y, ray_angle);

    // 2) NOW read the results of cast_ray
    int hit_type = last_hit_type;
    int wall_x   = last_wall_x;

    if (hit_type == 0) {
        // draw only ceiling/floor for this column
        for (int row = 0; row < RENDER_HEIGHT; ++row) {
            char pixel_char = ' ';
            unsigned char fg = COLOR_BLACK, bg = COLOR_BLACK;
            if (row < 2) bg = COLOR_DARK_BLUE;
            else if (row < 4) { pixel_char = '.'; fg = COLOR_DARK_GRAY; }
            else if (row >= RENDER_HEIGHT - 3) { pixel_char = '='; fg = COLOR_BROWN; }
            else if ((col + row) & 2) { pixel_char = '.'; fg = COLOR_DARK_GRAY; }
            text40_putchar_color(col, row + 2, pixel_char, fg, bg);
        }
        continue; // next column
    }



    depth_buffer[col] = dist;

    // 3) compute wall height, zero if no hit
    int numer = 160 * FP_SCALE;        // 160*16 = 2560 (16-bit safe)
    int wall_height = numer / (dist ? dist : 1);
    if (wall_height > RENDER_HEIGHT) wall_height = RENDER_HEIGHT;

    int wall_top    = (RENDER_HEIGHT - wall_height) / 2;
    int wall_bottom = wall_top + wall_height;

        // Draw column
        for (int row = 0; row < RENDER_HEIGHT; row++) {
            char pixel_char = ' ';
            unsigned char fg_color = COLOR_BLACK;
            unsigned char bg_color = COLOR_BLACK;
            
            if (row < wall_top) {
                // Ceiling - gradient effect
                if (row < 2) {
                    bg_color = COLOR_DARK_BLUE;
                } else if (row < 4) {
                    pixel_char = '.';
                    fg_color = COLOR_DARK_GRAY;
                }
            } else if (row >= wall_bottom) {
                // Floor - checkered pattern
                if ((col + row) & 2) {
                    pixel_char = '.';
                    fg_color = COLOR_DARK_GRAY;
                } else if (row > RENDER_HEIGHT - 3) {
                    pixel_char = '=';
                    fg_color = COLOR_BROWN;
                }
            } else {
                // Wall rendering
                int tex_y = ((row - wall_top) * TEX_HEIGHT) / wall_height;
                int tex_x = (wall_x * TEX_WIDTH) >> FP_SHIFT;
                tex_x = tex_x & 7;
                tex_y = tex_y & 7;
                int tex_idx = tex_y * TEX_WIDTH + tex_x;
                
// choose texture strictly by hit_type
unsigned char tex_val;
if (hit_type == 2)      tex_val = tex_tech[tex_idx];   // door
else if (hit_type == 3) tex_val = tex_stone[tex_idx];  // key wall
else                    tex_val = tex_brick[tex_idx];  // normal wall
               if (hit_type == 0) {
                   // no wall: render only ceiling/floor
                   // set wall_height to 0 so the “wall” branch is skipped
                   wall_height = 0;
               }

// Tuned for Q12.4: near/mid/far at ~4/8/12 tiles
int near_d = 4  * FP_SCALE;   // 64
int mid_d  = 8  * FP_SCALE;   // 128
int far_d  = 12 * FP_SCALE;   // 192

if (dist < near_d) {
    if (tex_val) { pixel_char = '#'; fg_color = COLOR_WHITE;  bg_color = COLOR_LIGHT_GRAY; }
    else         { pixel_char = '='; fg_color = COLOR_LIGHT_GRAY; bg_color = COLOR_DARK_GRAY; }
} else if (dist < mid_d) {
    if (tex_val) { pixel_char = '+'; fg_color = COLOR_LIGHT_GRAY; bg_color = COLOR_DARK_GRAY; }
    else         { pixel_char = '-'; fg_color = COLOR_DARK_GRAY;  bg_color = COLOR_BLACK;    }
} else if (dist < far_d) {
    // Make far walls brighter than floor dots so they stand out
    pixel_char = tex_val ? ':' : '.';
    fg_color = COLOR_LIGHT_GRAY;      // <- was DARK_GRAY (looked like floor)
    bg_color = COLOR_BLACK;
} else {
    pixel_char = '.';                  // super far
    fg_color = COLOR_DARK_GRAY;
    bg_color = COLOR_BLACK;
}

                if (last_hit_side) {
                    // horizontal faces a touch darker
                    if (fg_color == COLOR_WHITE) fg_color = COLOR_LIGHT_GRAY;
                    else if (fg_color == COLOR_LIGHT_GRAY) fg_color = COLOR_DARK_GRAY;
                }
                
                // Special coloring for doors
                if (hit_type == 2) {
                    if (fg_color == COLOR_WHITE) fg_color = COLOR_YELLOW;
                    if (fg_color == COLOR_LIGHT_GRAY) fg_color = COLOR_BROWN;
                }
            }
            
            text40_putchar_color(col, row + 2, pixel_char, fg_color, bg_color);

            // Draw some debug markers to see where we hit a wall
//            text40_putchar_color(col, 1, hit_type ? '|' : '.', COLOR_GREEN, COLOR_BLACK);
//            text40_putchar_color(col, 2, (wall_height>0)?'|':'_', COLOR_GREEN, COLOR_BLACK);

        }
    }
    
    // Render enemy sprite if visible
    if (enemy_active) {
        // Calculate enemy position relative to player
        int dx = enemy_x - player_x;
        int dy = enemy_y - player_y;

        // 16-bit safe: 0..~448 in this map
        int enemy_dist = abs_val(dx) + abs_val(dy);


        // Calculate angle to enemy
        int angle_to_enemy = 0;
        if (dx != 0 || dy != 0) {
            // Simple atan2 approximation
            for (int a = 0; a < 256; a++) {
                int test_x = fp_mul(enemy_dist, cos_lookup(a));
                int test_y = fp_mul(enemy_dist, sin_lookup(a));
                if (abs_val(test_x - dx) < 50 && abs_val(test_y - dy) < 50) {
                    angle_to_enemy = a;
                    break;
                }
            }
        }
        
        // Check if enemy is in view
        int angle_diff = ((angle_to_enemy - player_angle + 128) & 255) - 128;
        if (abs_val(angle_diff) < 45 && enemy_dist < MAX_DIST / 2) {
            // Calculate screen position
            int screen_x = SCREEN_WIDTH / 2 + (angle_diff * SCREEN_WIDTH) / 90;
            // sprite size keeps similar feel (tweak the “+1” or the divisor to taste)
            int sprite_size = (FP_SCALE * 12) / ((enemy_dist >> 1) + 1);
            if (sprite_size > RENDER_HEIGHT) sprite_size = RENDER_HEIGHT;
            
            if (sprite_size > 2) {
                if (screen_x >= 0) {
                    if (screen_x < SCREEN_WIDTH) {
                        int sprite_top = (RENDER_HEIGHT - sprite_size) / 2;
                        
                        // Draw enemy sprite
                        int max_sy = sprite_size;
                        if (max_sy > TEX_HEIGHT) max_sy = TEX_HEIGHT;
                        for (int sy = 0; sy < max_sy; sy++) {
                            for (int sx = -sprite_size/2; sx < sprite_size/2; sx++) {
                                int screen_col = screen_x + sx;
                                if (screen_col >= 0 && screen_col < SCREEN_WIDTH) {
                                    // Check depth buffer
                                    if (enemy_dist < depth_buffer[screen_col]) {
                                        int tex_x = ((sx + sprite_size/2) * TEX_WIDTH) / sprite_size;
                                        int tex_y = (sy * TEX_HEIGHT) / sprite_size;
                                        if (tex_x >= 0) {
                                            if (tex_x < TEX_WIDTH) {
                                                if (tex_y >= 0) {
                                                    if (tex_y < TEX_HEIGHT) {
                                                        int tex_idx = tex_y * TEX_WIDTH + tex_x;
                                                        if (sprite_enemy[tex_idx]) {
                                                            int screen_row = sprite_top + sy;
                                                            if (screen_row >= 0) {
                                                                if (screen_row < RENDER_HEIGHT) {
                                                                    // Red enemy with shading
                                                                    char enemy_char = '@';
                                                                    unsigned char enemy_color = COLOR_RED;
                                                                    if (enemy_dist > FP_SCALE * 3) {
                                                                        enemy_char = 'o';
                                                                        enemy_color = COLOR_BROWN;
                                                                    }
                                                                    text40_putchar_color(screen_col, screen_row + 2, 
                                                                                       enemy_char, enemy_color, COLOR_BLACK);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Draw weapon sprite at bottom
    for (int y = 0; y < 5; y++) {
        for (int x = 0; x < 8; x++) {
            if (sprite_weapon[y * 8 + x]) {
                text40_putchar_color(SCREEN_WIDTH/2 - 4 + x, RENDER_HEIGHT - 5 + y + 2,
                                   '=', COLOR_DARK_GRAY, COLOR_BLACK);
            }
        }
    }
    
    // Muzzle flash effect
    if (frame_count % 20 < 2) {
        text40_putchar_color(SCREEN_WIDTH/2, RENDER_HEIGHT - 3, '*', COLOR_YELLOW, COLOR_ORANGE);
    }
}


// Draw the HUD
void draw_hud() {
    // Top status bar
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, 0, ' ', COLOR_WHITE, COLOR_DARK_GRAY);
    }
    text40_puts_color(1, 0, "DOOM CLONE", COLOR_RED, COLOR_DARK_GRAY);

    // Health bar
    text40_puts_color(15, 0, "HP:", COLOR_RED, COLOR_DARK_GRAY);
    for (int i = 0; i < 10; i++) {
        if (i < player_health / 10) {
            text40_putchar_color(18 + i, 0, '=', COLOR_RED, COLOR_DARK_GRAY);
        } else {
            text40_putchar_color(18 + i, 0, '-', COLOR_DARK_GRAY, COLOR_BLACK);
        }
    }

    // Ammo counter
    text40_puts_color(30, 0, "AMMO:", COLOR_YELLOW, COLOR_DARK_GRAY);
    text40_putchar_color(35, 0, '0' + (player_ammo / 10), COLOR_YELLOW, COLOR_DARK_GRAY);
    text40_putchar_color(36, 0, '0' + (player_ammo % 10), COLOR_YELLOW, COLOR_DARK_GRAY);

    // Bottom status bar
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, 23, '=', COLOR_DARK_GRAY, COLOR_BLACK);
    }

    // Position display
    int map_x = player_x >> FP_SHIFT;
    int map_y = player_y >> FP_SHIFT;
    text40_puts_color(1, 24, "X:", COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(3, 24, '0' + map_x / 10, COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(4, 24, '0' + map_x % 10, COLOR_GREEN, COLOR_BLACK);

    // after printing X: and the two digits at cols 3–4
    int xf = ((player_x & (FP_SCALE - 1)) * 10) / FP_SCALE;   // 0..9
    text40_putchar_color(5, 24, '.', COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(6, 24, '0' + xf, COLOR_GREEN, COLOR_BLACK);

    text40_puts_color(7, 24, "Y:", COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(9, 24, '0' + map_y / 10, COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(10, 24, '0' + map_y % 10, COLOR_GREEN, COLOR_BLACK);

    // after Y digits at cols 9–10
    int yf = ((player_y & (FP_SCALE - 1)) * 10) / FP_SCALE;
    text40_putchar_color(11, 24, '.', COLOR_GREEN, COLOR_BLACK);
    text40_putchar_color(12, 24, '0' + yf, COLOR_GREEN, COLOR_BLACK);


    // Direction compass
    char dir_char;
    int a = player_angle & 255;
    if (a >= 224 || a < 32)      dir_char = 'E';
    else if (a < 96)             dir_char = 'N';
    else if (a < 160)            dir_char = 'W';
    else                         dir_char = 'S';

    text40_puts_color(14, 24, "DIR:", COLOR_BLUE, COLOR_BLACK);
    text40_putchar_color(18, 24, dir_char, COLOR_BLUE, COLOR_BLACK);

    char * state = player_state == S_MOVE_FORWARD ? "F" : "S";

    // Score/kills
    text40_puts_color(22, 24, state, COLOR_WHITE, COLOR_BLACK);
//    text40_puts_color(28, 24, "000", COLOR_WHITE, COLOR_BLACK);

    // Status messages
    if (frame_count < 50) {
        text40_puts_color(33, 24, "FIND EXIT", COLOR_YELLOW, COLOR_BLACK);
    }

}

void spawn_on_open_tile(void) {
    int mx = player_x >> FP_SHIFT, my = player_y >> FP_SHIFT;
    if (get_map(mx, my) == 0) return;
    for (int r = 1; r <= 6; ++r) {
        for (int dy = -r; dy <= r; ++dy)
        for (int dx = -r; dx <= r; ++dx) {
            int nx = mx + dx, ny = my + dy;
            if (get_map(nx, ny) == 0) {
                player_x = nx*FP_SCALE + FP_HALF;
                player_y = ny*FP_SCALE + FP_HALF;
                return;
            }
        }
    }
}

// Initialize game
void init_game() {
    // Place player in starting position
    player_x = 1*FP_SCALE + FP_HALF;  // (1,1) center
        player_y = 1*FP_SCALE + FP_HALF;
        player_angle = 0;
        player_health = 100;
        player_ammo = 50;
        frame_count = 0;

        enemy_x = 8 * FP_SCALE;
        enemy_y = 8 * FP_SCALE;
        enemy_active = 1;
        enemy_health = 30;
        enemy_angle = 128;

        spawn_on_open_tile();
}

void clamp_player(void){
    // Keep inside the map with room for collision radius
    // We need to stay away from the walls at tiles 0 and 15
    int min  = 1*FP_SCALE + PLAYER_R + 1;           // Just over 1 tile + radius
    int maxx = (MAP_WIDTH  - 2)*FP_SCALE - PLAYER_R - 1;  // Stay away from tile 15
    int maxy = (MAP_HEIGHT - 2)*FP_SCALE - PLAYER_R - 1;  // 14*16 - 3 - 1 = 220 (13.75 tiles)

    if (player_x < min)  player_x = min;
    if (player_x > maxx) player_x = maxx;
    if (player_y < min)  player_y = min;
    if (player_y > maxy) player_y = maxy;
}

int free_circle(int x, int y){
    int tx = x >> FP_SHIFT, ty = y >> FP_SHIFT;
    // walls are solid at tile centers; check the four faces
    if (get_map((x + PLAYER_R) >> FP_SHIFT, ty)) return 0; // east
    if (get_map((x - PLAYER_R) >> FP_SHIFT, ty)) return 0; // west
    if (get_map(tx, (y + PLAYER_R) >> FP_SHIFT)) return 0; // south
    if (get_map(tx, (y - PLAYER_R) >> FP_SHIFT)) return 0; // north
    return 1;
}

int can_move_forward(void){
    int PROBE = FP_SCALE/2; // 8 subunits (0.5 tile); can be FP_SCALE/4 too
    int dx = cos_lookup(player_angle);
    int dy = sin_lookup(player_angle);

    // look a little ahead, *plus* your radius in the facing direction
    int nx = player_x + fp_mul(PROBE, dx) + (dx >= 0 ? PLAYER_R : -PLAYER_R);
    int ny = player_y + fp_mul(PROBE, dy) + (dy >= 0 ? PLAYER_R : -PLAYER_R);

    int mx = nx >> FP_SHIFT, my = ny >> FP_SHIFT;

    // keep within the 14x14 inner box
    if (mx < 1 || mx > 14 || my < 1 || my > 14) return 0;

    // tile must be empty, and the circle must fit
    return get_map(mx, my) == 0 && free_circle(nx, ny);
}

void draw_debug() {
   int step_x = fp_mul(MOVE_SPEED, cos_lookup(player_angle));
       int step_y = fp_mul(MOVE_SPEED, sin_lookup(player_angle));

       int new_x = player_x + step_x;
       int new_y = player_y + step_y;

       // Check if the new position is a wall
       int mx = new_x >> FP_SHIFT;
       int my = new_y >> FP_SHIFT;

       // we want to draw mx and my in the debug info
       char * c_mx = "  ";
         if (mx < 10) c_mx[0] = '0' + mx;
            else if (mx < 100) { c_mx[0] = '0' + (mx / 10); c_mx[1] = '0' + (mx % 10); }
            else { c_mx[0] = '1'; c_mx[1] = '5'; } // max 15 tiles

         char * c_my = "  ";
            if (my < 10) c_my[0] = '0' + my;
                else if (my < 100) { c_my[0] = '0' + (my / 10); c_my[1] = '0' + (my % 10); }
                else { c_my[0] = '1'; c_my[1] = '5'; } // max 15 tiles

       text40_puts_color(0, 1, "DEBUG:", COLOR_BLUE, COLOR_BLACK);
       text40_puts_color(0, 1, c_mx, COLOR_BLUE, COLOR_BLACK);
       text40_puts_color(3, 1, c_my, COLOR_BLUE, COLOR_BLACK);
}

void move_forward(void) {
    // Move player forward if possible
    if (can_move_forward()) {
        int step_x = fp_mul(MOVE_SPEED, cos_lookup(player_angle));
        int step_y = fp_mul(MOVE_SPEED, sin_lookup(player_angle));

        // Check if step forward after this leaves player in a wall:
        int next_x = player_x + step_x;
        int next_y = player_y + step_y;

        player_x += step_x;
        player_y += step_y;

    } else {
        // If stuck, switch to rotate state
        player_state = S_ROTATE_UNTIL_FREE;
    }
}

void handle_player_movement(void) {
    // Handle player movement based on state
    if (player_state == S_MOVE_FORWARD) {
        move_forward();
    } else if (player_state == S_ROTATE_UNTIL_FREE) {
        player_angle = (player_angle - TURN_SPEED) & 255; // Turn left

        // If unstuck (can move forward), switch back to moving state:
        if (can_move_forward()) {
            player_state = S_MOVE_FORWARD;
        }
    }
}

// keep your clamp_player() as you fixed it earlier



// Handle input (simulated for demo)
//void handle_input_orig() {
//    // Random movement for demo
////    int action = rand_range(100);
////    player_angle = (player_angle + 2) & 255;
////    int action = 0;
//
////    player_angle = 0;         // face east
////    move_step(FP_SCALE/2);
////    player_x += 1;
////    return;
//
//    int action = 0;                 // keep your “always forward” debug
//    player_angle = (player_angle + 2) & 255;
//
//    if (action < 15)            move_step(MOVE_SPEED);
//    else if (action < 25)       move_step(-MOVE_SPEED);
//    else if (action < 30)       player_angle = (player_angle - TURN_SPEED) & 255;
//    else if (action < 35)       player_angle = (player_angle + TURN_SPEED) & 255;
//    else if (action < 40) {     // strafe left
//        int a = (player_angle - 64) & 255;
//        int old = player_angle; player_angle = a; move_step(MOVE_SPEED/2); player_angle = old;
//    }
//    else if (action < 45) {     // strafe right
//        int a = (player_angle + 64) & 255;
//        int old = player_angle; player_angle = a; move_step(MOVE_SPEED/2); player_angle = old;
//    } else if (action < 48 && player_ammo > 0) {
//        // Shoot
//        player_ammo--;
//    }
//
//    // Move enemy (simple AI)
//    if (enemy_active && rand_range(100) < 30) {
//        // Move towards player sometimes
//        int dx = player_x - enemy_x;
//        int dy = player_y - enemy_y;
//
//        if (abs_val(dx) > abs_val(dy)) {
//            // Move horizontally
//            if (dx > 0) {
//                enemy_x += FP_SCALE / 8;
//            } else {
//                enemy_x -= FP_SCALE / 8;
//            }
//        } else {
//            // Move vertically
//            if (dy > 0) {
//                enemy_y += FP_SCALE / 8;
//            } else {
//                enemy_y -= FP_SCALE / 8;
//            }
//        }
//
//        // Random movement
//        if (rand_range(100) < 40) {
//            enemy_angle = (enemy_angle + rand_range(64) - 32) & 255;
//            int move_x = enemy_x + fp_mul(FP_SCALE / 4, cos_lookup(enemy_angle));
//            int move_y = enemy_y + fp_mul(FP_SCALE / 4, sin_lookup(enemy_angle));
//
//            if (get_map(move_x >> FP_SHIFT, move_y >> FP_SHIFT) == 0) {
//                enemy_x = move_x;
//                enemy_y = move_y;
//            }
//        }
//    }
//}

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
    
    // Initialize game
    init_game();
    
    // Main game loop
    for (frame_count = 0; frame_count < 600; frame_count++) {
        // Clear display
        display_clear();
        
        // Handle input
        handle_player_movement();
        
        // Render 3D view
        render_3d();
        
        // Draw HUD
        draw_hud();

        draw_debug();
        
        // Update display
        display_flush();
        
        // Frame delay
        delay(2500);
        
        // Random events
        if (rand_range(100) < 3) {
            // Pickup ammo
            player_ammo += 10;
            if (player_ammo > 99) player_ammo = 99;
        }
        
        if (rand_range(100) < 2) {
            // Take damage
            player_health -= 5;
            if (player_health <= 0) {
                // Death screen
                for (int y = 0; y < SCREEN_HEIGHT; y++) {
                    for (int x = 0; x < SCREEN_WIDTH; x++) {
                        text40_putchar_color(x, y, ' ', COLOR_BLACK, COLOR_RED);
                    }
                }
                text40_puts_color(14, 12, " YOU DIED ", COLOR_WHITE, COLOR_RED);
                display_flush();
                delay(50000);
                break;
            }
        }
        
        // Victory condition - reach far corner
//        int px = player_x >> FP_SHIFT;
//        int py = player_y >> FP_SHIFT;
//        if (px >= 14) {
//            if (py >= 14) {
//                text40_puts_color(12, 12, " LEVEL COMPLETE ", COLOR_BLACK, COLOR_GREEN);
//                display_flush();
//                delay(50000);
//                break;
//            }
//        }
    }
    
    // End screen
    if (frame_count >= 1600) {
        text40_puts_color(10, 12, " DEMO COMPLETE ", COLOR_WHITE, COLOR_GREEN);
        display_flush();
        delay(30000);
    }
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Doom clone demo completed!");
    
    return 0;
}