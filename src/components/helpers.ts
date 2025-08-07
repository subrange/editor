/**
 * Measure the width of a character in the monospace font used by the editor
 * @returns The width of a single character in pixels
 */
export function measureCharacterWidth(): number {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
        throw new Error("Failed to get canvas context");
    }
    context.font = "14px monospace"; // Match font-mono text-sm
    const width = context.measureText("M").width;
    return width;
}

// Export as a constant that can be calculated once
export const CHAR_WIDTH = measureCharacterWidth();