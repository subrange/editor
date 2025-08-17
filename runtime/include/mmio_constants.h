#ifndef MMIO_CONSTANTS_H
#define MMIO_CONSTANTS_H

// MMIO Header Addresses (bank 0, words 0..31)
#define MMIO_TTY_OUT       0
#define MMIO_TTY_STATUS    1
#define MMIO_TTY_IN_POP    2
#define MMIO_TTY_IN_STATUS 3
#define MMIO_RNG           4
#define MMIO_RNG_SEED      5
#define MMIO_DISP_MODE     6
#define MMIO_DISP_STATUS   7
#define MMIO_DISP_CTL      8
#define MMIO_DISP_FLUSH    9

// Display modes
#define DISP_MODE_OFF    0
#define DISP_MODE_TTY    1
#define DISP_MODE_TEXT40 2

// Display control bits
#define DISP_CTL_ENABLE (1 << 0)
#define DISP_CTL_CLEAR  (1 << 1)

// Display status bits
#define DISP_STATUS_READY      (1 << 0)
#define DISP_STATUS_FLUSH_DONE (1 << 1)

// TTY status bits
#define TTY_STATUS_READY    (1 << 0)
#define TTY_STATUS_HAS_BYTE (1 << 0)

// TEXT40 display constants
#define TEXT40_BASE   32
#define TEXT40_WIDTH  40
#define TEXT40_HEIGHT 25
#define TEXT40_SIZE   1000

// Theme color indices (PICO-8 palette)
#define COLOR_BLACK        0
#define COLOR_DARK_BLUE    1
#define COLOR_DARK_PURPLE  2
#define COLOR_DARK_GREEN   3
#define COLOR_BROWN        4
#define COLOR_DARK_GRAY    5
#define COLOR_LIGHT_GRAY   6
#define COLOR_WHITE        7
#define COLOR_RED          8
#define COLOR_ORANGE       9
#define COLOR_YELLOW       10
#define COLOR_GREEN        11
#define COLOR_BLUE         12
#define COLOR_INDIGO       13
#define COLOR_PINK         14
#define COLOR_PEACH        15

// Helper macro to create attribute byte from foreground and background colors
#define MAKE_ATTR(fg, bg) (((bg) << 4) | (fg))

#endif // MMIO_CONSTANTS_H