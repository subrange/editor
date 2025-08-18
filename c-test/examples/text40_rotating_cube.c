// 3D Rotating Cube Demo using Fixed-Point Math
// Renders a wireframe cube with rotation using the qfixed library

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>
#include <qfixed.h>

#define SCREEN_WIDTH 40
#define SCREEN_HEIGHT 25

// Cube vertices (8 vertices, each with x,y,z coordinates)
// Using 1D array: vertices[vertex_index * 3 + coordinate]
// Cube centered at origin with size 1.0 (in fixed point)
struct q16_16_t vertices[24]; // 8 vertices * 3 coordinates

// Transformed vertices after rotation
struct q16_16_t transformed[24];

// Screen coordinates after projection
int screen_x[8];
int screen_y[8];

// Rotation angles
struct q16_16_t angle_x;
struct q16_16_t angle_y;
struct q16_16_t angle_z;

// Edge list - pairs of vertex indices that form edges
// 12 edges * 2 vertices per edge
int edges[24] = {
    0, 1,  1, 2,  2, 3,  3, 0,  // Front face
    4, 5,  5, 6,  6, 7,  7, 4,  // Back face  
    0, 4,  1, 5,  2, 6,  3, 7   // Connecting edges
};

// Z-buffer for simple depth sorting (using 1D array)
unsigned char zbuffer[1000]; // 40*25 = 1000

// Initialize cube vertices
void init_cube() {
    // Initialize vertices for a unit cube centered at origin
    // Using temporary variables to avoid struct initializer issues
    struct q16_16_t minus_one, plus_one;
    minus_one.integer = -1;
    minus_one.frac = 0;
    plus_one.integer = 1;
    plus_one.frac = 0;
    
    // Front face (z = -1)
    vertices[0] = minus_one;  // v0: x = -1
    vertices[1] = minus_one;  // v0: y = -1
    vertices[2] = minus_one;  // v0: z = -1
    
    vertices[3] = plus_one;   // v1: x = 1
    vertices[4] = minus_one;  // v1: y = -1
    vertices[5] = minus_one;  // v1: z = -1
    
    vertices[6] = plus_one;   // v2: x = 1
    vertices[7] = plus_one;   // v2: y = 1
    vertices[8] = minus_one;  // v2: z = -1
    
    vertices[9] = minus_one;  // v3: x = -1
    vertices[10] = plus_one;  // v3: y = 1
    vertices[11] = minus_one; // v3: z = -1
    
    // Back face (z = 1)
    vertices[12] = minus_one; // v4: x = -1
    vertices[13] = minus_one; // v4: y = -1
    vertices[14] = plus_one;  // v4: z = 1
    
    vertices[15] = plus_one;  // v5: x = 1
    vertices[16] = minus_one; // v5: y = -1
    vertices[17] = plus_one;  // v5: z = 1
    
    vertices[18] = plus_one;  // v6: x = 1
    vertices[19] = plus_one;  // v6: y = 1
    vertices[20] = plus_one;  // v6: z = 1
    
    vertices[21] = minus_one; // v7: x = -1
    vertices[22] = plus_one;  // v7: y = 1
    vertices[23] = plus_one;  // v7: z = 1
    
    // Initialize rotation angles
    angle_x.integer = 0;
    angle_x.frac = 0;
    angle_y.integer = 0;
    angle_y.frac = 0;
    angle_z.integer = 0;
    angle_z.frac = 0;
}

// Rotate vertices around all three axes
void rotate_cube() {
    struct q16_16_t sin_x, cos_x;
    struct q16_16_t sin_y, cos_y;
    struct q16_16_t sin_z, cos_z;
    
    // Calculate sin/cos for each angle
    q_sin(&sin_x, &angle_x);
    q_cos(&cos_x, &angle_x);
    q_sin(&sin_y, &angle_y);
    q_cos(&cos_y, &angle_y);
    q_sin(&sin_z, &angle_z);
    q_cos(&cos_z, &angle_z);
    
    // Apply rotation to each vertex
    for (int i = 0; i < 8; i++) {
        // Get original vertex coordinates
        struct q16_16_t x = vertices[i * 3 + 0];
        struct q16_16_t y = vertices[i * 3 + 1];
        struct q16_16_t z = vertices[i * 3 + 2];
        
        // Rotation around X axis
        struct q16_16_t temp_y, temp_z;
        struct q16_16_t y_cos, z_sin, y_sin, z_cos;
        
        q_mul(&y_cos, &y, &cos_x);
        q_mul(&z_sin, &z, &sin_x);
        q_sub(&temp_y, &y_cos, &z_sin);
        
        q_mul(&y_sin, &y, &sin_x);
        q_mul(&z_cos, &z, &cos_x);
        q_add(&temp_z, &y_sin, &z_cos);
        
        y = temp_y;
        z = temp_z;
        
        // Rotation around Y axis
        struct q16_16_t temp_x;
        struct q16_16_t x_cos, z_sin2, x_sin, z_cos2;
        
        q_mul(&x_cos, &x, &cos_y);
        q_mul(&z_sin2, &z, &sin_y);
        q_add(&temp_x, &x_cos, &z_sin2);
        
        q_mul(&x_sin, &x, &sin_y);
        q_mul(&z_cos2, &z, &cos_y);
        q_sub(&temp_z, &z_cos2, &x_sin);
        
        x = temp_x;
        z = temp_z;
        
        // Rotation around Z axis
        struct q16_16_t x_cos2, y_sin2, x_sin2, y_cos2;
        
        q_mul(&x_cos2, &x, &cos_z);
        q_mul(&y_sin2, &y, &sin_z);
        q_sub(&temp_x, &x_cos2, &y_sin2);
        
        q_mul(&x_sin2, &x, &sin_z);
        q_mul(&y_cos2, &y, &cos_z);
        q_add(&temp_y, &x_sin2, &y_cos2);
        
        // Store transformed vertices
        transformed[i * 3 + 0] = temp_x;
        transformed[i * 3 + 1] = temp_y;
        transformed[i * 3 + 2] = z;
    }
}

// Project 3D coordinates to 2D screen
void project_vertices() {
    struct q16_16_t distance;
    distance.integer = 5;  // Distance from viewer
    distance.frac = 0;
    
    struct q16_16_t scale;
    scale.integer = 8;  // Scale factor for display
    scale.frac = 0;
    
    for (int i = 0; i < 8; i++) {
        struct q16_16_t x = transformed[i * 3 + 0];
        struct q16_16_t y = transformed[i * 3 + 1];
        struct q16_16_t z = transformed[i * 3 + 2];
        
        // Add distance to z to move cube away from viewer
        struct q16_16_t z_dist;
        q_add(&z_dist, &z, &distance);
        
        // Perspective projection: x' = x/z, y' = y/z
        struct q16_16_t proj_x, proj_y;
        
        // Avoid division by very small numbers
        if (z_dist.integer > 0 || (z_dist.integer == 0 && z_dist.frac > 0x1000)) {
            q_div(&proj_x, &x, &z_dist);
            q_div(&proj_y, &y, &z_dist);
        } else {
            proj_x = x;
            proj_y = y;
        }
        
        // Scale and convert to screen coordinates
        struct q16_16_t scaled_x, scaled_y;
        q_mul(&scaled_x, &proj_x, &scale);
        q_mul(&scaled_y, &proj_y, &scale);
        
        // Convert to integer and center on screen
        screen_x[i] = q_to_int(&scaled_x) + SCREEN_WIDTH / 2;
        screen_y[i] = q_to_int(&scaled_y) + SCREEN_HEIGHT / 2;
        
        // Clamp to screen bounds
        if (screen_x[i] < 0) screen_x[i] = 0;
        if (screen_x[i] >= SCREEN_WIDTH) screen_x[i] = SCREEN_WIDTH - 1;
        if (screen_y[i] < 0) screen_y[i] = 0;
        if (screen_y[i] >= SCREEN_HEIGHT) screen_y[i] = SCREEN_HEIGHT - 1;
    }
}

// Draw a line between two points using Bresenham's algorithm
void draw_line(int x0, int y0, int x1, int y1, char ch, unsigned char color) {
    int dx = x1 - x0;
    int dy = y1 - y0;
    
    // Handle negative deltas
    int sx = 1;
    int sy = 1;
    
    if (dx < 0) {
        dx = -dx;
        sx = -1;
    }
    if (dy < 0) {
        dy = -dy;
        sy = -1;
    }
    
    int err = dx - dy;
    
    while (1) {
        // Draw pixel
        if (x0 >= 0 && x0 < SCREEN_WIDTH && y0 >= 0 && y0 < SCREEN_HEIGHT) {
            text40_putchar_color(x0, y0, ch, color, COLOR_BLACK);
        }
        
        // Check if we've reached the end
        if (x0 == x1 && y0 == y1) break;
        
        int e2 = 2 * err;
        
        if (e2 > -dy) {
            err -= dy;
            x0 += sx;
        }
        
        if (e2 < dx) {
            err += dx;
            y0 += sy;
        }
    }
}

// Render the cube
void render_cube() {
    // Clear screen
    display_clear();
    
    // Draw title
    text40_puts_color(12, 1, "ROTATING CUBE", COLOR_YELLOW, COLOR_DARK_BLUE);
    text40_puts_color(10, 2, "Fixed-Point Demo", COLOR_WHITE, COLOR_BLACK);
    
    // Draw all edges
    for (int i = 0; i < 12; i++) {
        int v0 = edges[i * 2];
        int v1 = edges[i * 2 + 1];
        
        // Choose color based on edge type
        unsigned char color = COLOR_GREEN;
        if (i < 4) {
            color = COLOR_WHITE;  // Front face
        } else if (i < 8) {
            color = COLOR_LIGHT_GRAY;  // Back face
        } else {
            color = COLOR_BLUE;  // Connecting edges
        }
        
        draw_line(screen_x[v0], screen_y[v0], 
                 screen_x[v1], screen_y[v1], '*', color);
    }
    
    // Draw vertices as bright points
    for (int i = 0; i < 8; i++) {
        text40_putchar_color(screen_x[i], screen_y[i], 'O', COLOR_YELLOW, COLOR_BLACK);
    }
    
    // Draw info
    text40_puts_color(1, SCREEN_HEIGHT - 2, "Angle X:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(10, SCREEN_HEIGHT - 2, '0' + (angle_x.integer % 10), COLOR_GREEN, COLOR_BLACK);
    
    text40_puts_color(15, SCREEN_HEIGHT - 2, "Y:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(18, SCREEN_HEIGHT - 2, '0' + (angle_y.integer % 10), COLOR_GREEN, COLOR_BLACK);
    
    text40_puts_color(22, SCREEN_HEIGHT - 2, "Z:", COLOR_WHITE, COLOR_BLACK);
    text40_putchar_color(25, SCREEN_HEIGHT - 2, '0' + (angle_z.integer % 10), COLOR_GREEN, COLOR_BLACK);
    
    // Draw border
    for (int x = 0; x < SCREEN_WIDTH; x++) {
        text40_putchar_color(x, 0, '-', COLOR_DARK_GRAY, COLOR_BLACK);
        text40_putchar_color(x, SCREEN_HEIGHT - 1, '-', COLOR_DARK_GRAY, COLOR_BLACK);
    }
    for (int y = 0; y < SCREEN_HEIGHT; y++) {
        text40_putchar_color(0, y, '|', COLOR_DARK_GRAY, COLOR_BLACK);
        text40_putchar_color(SCREEN_WIDTH - 1, y, '|', COLOR_DARK_GRAY, COLOR_BLACK);
    }
}

// Simple delay function
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
    
    // Initialize cube
    init_cube();
    
    // Animation parameters
    struct q16_16_t angle_increment;
    angle_increment.integer = 0;
    angle_increment.frac = 0x2000;  // Small rotation increment
    
    // Main animation loop
    for (int frame = 0; frame < 200; frame++) {
        // Rotate the cube
        rotate_cube();
        
        // Project to 2D
        project_vertices();
        
        // Render
        render_cube();
        
        // Show frame number
        text40_putchar_color(SCREEN_WIDTH - 4, 1, '0' + ((frame / 100) % 10), COLOR_WHITE, COLOR_BLACK);
        text40_putchar_color(SCREEN_WIDTH - 3, 1, '0' + ((frame / 10) % 10), COLOR_WHITE, COLOR_BLACK);
        text40_putchar_color(SCREEN_WIDTH - 2, 1, '0' + (frame % 10), COLOR_WHITE, COLOR_BLACK);
        
        display_flush();
        delay(5000);
        
        // Update rotation angles
        q_add(&angle_x, &angle_x, &angle_increment);
        
        // Slower rotation on Y axis
        if (frame % 2 == 0) {
            q_add(&angle_y, &angle_y, &angle_increment);
        }
        
        // Even slower rotation on Z axis
        if (frame % 3 == 0) {
            q_add(&angle_z, &angle_z, &angle_increment);
        }
        
        // Keep angles in reasonable range
        if (angle_x.integer > 6) {
            angle_x.integer = 0;
        }
        if (angle_y.integer > 6) {
            angle_y.integer = 0;
        }
        if (angle_z.integer > 6) {
            angle_z.integer = 0;
        }
    }
    
    // End sequence
    display_clear();
    text40_puts_color(11, 11, "DEMO COMPLETE", COLOR_WHITE, COLOR_GREEN);
    text40_puts_color(12, 13, "EXITING...", COLOR_BLACK, COLOR_YELLOW);
    display_flush();
    delay(5000);
    
    // Return to normal
    display_set_mode(DISP_MODE_OFF);
    puts("Rotating cube demo complete!");
    
    return 0;
}