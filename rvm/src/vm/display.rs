use crate::constants::*;
use crate::display_rgb565::RGB565Display;
use super::VM;
use crossterm::{
    cursor,
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor},
    ExecutableCommand,
};
use std::io::{self, Write};

impl VM {
    /// Calculate the data section offset based on display mode and resolution
    pub(super) fn get_data_section_offset(&self) -> usize {
        match self.display_mode {
            DISP_RGB565 => {
                // For RGB565, data section must come after both framebuffers
                let width = ((self.display_resolution >> 8) & 0xFF) as usize;
                let height = (self.display_resolution & 0xFF) as usize;
                
                if width > 0 && height > 0 {
                    let pixels_per_buffer = width * height;
                    let total_framebuffer_words = pixels_per_buffer * 2; // Double-buffered
                    
                    // Data section starts after MMIO headers (32 words) and both framebuffers
                    32 + total_framebuffer_words
                } else {
                    // Invalid resolution, use default
                    TEXT40_LAST_WORD + 1
                }
            },
            DISP_TEXT40 => {
                // For TEXT40, data section starts after VRAM (word 1032)
                TEXT40_LAST_WORD + 1
            },
            _ => {
                // For OFF or TTY modes, use minimal offset (after MMIO headers)
                // But keep compatibility with existing binaries by using TEXT40 offset
                TEXT40_LAST_WORD + 1
            }
        }
    }
    
    /// Check if data section will be affected by display mode change
    pub(super) fn check_data_section_conflict(&self, new_mode: u16) -> bool {
        if new_mode == DISP_RGB565 {
            let width = ((self.display_resolution >> 8) & 0xFF) as usize;
            let height = (self.display_resolution & 0xFF) as usize;
            
            if width > 0 && height > 0 {
                let pixels_per_buffer = width * height;
                let total_framebuffer_words = pixels_per_buffer * 2;
                let required_end = 32 + total_framebuffer_words;
                
                // Check if current data at word 1032 would conflict
                if required_end > TEXT40_LAST_WORD + 1 {
                    eprintln!("WARNING: RGB565 mode {}x{} requires {} words for framebuffers", 
                             width, height, total_framebuffer_words);
                    eprintln!("         This conflicts with data section at word {}", TEXT40_LAST_WORD + 1);
                    eprintln!("         Data section should start at word {} or later", required_end);
                    return true;
                }
            }
        }
        false
    }
    
    /// Handle display mode changes
    pub(super) fn handle_display_mode_change(&mut self, new_mode: u16) {
        // Exit current mode
        match self.display_mode {
            DISP_TTY if self.terminal_raw_mode => {
                self.exit_tty_mode();
            },
            DISP_TEXT40 if self.terminal_raw_mode => {
                self.exit_text40_mode();
            },
            DISP_RGB565 => {
                // Shutdown RGB565 display
                if let Some(mut display) = self.rgb565_display.take() {
                    display.shutdown();
                }
            },
            _ => {}
        }
        
        // Enter new mode
        match new_mode {
            DISP_TTY if !self.terminal_raw_mode => {
                self.enter_tty_mode();
            },
            DISP_TEXT40 if !self.terminal_raw_mode => {
                self.enter_text40_mode();
            },
            DISP_RGB565 => {
                // Initialize RGB565 display
                let width = ((self.display_resolution >> 8) & 0xFF) as u8;
                let height = (self.display_resolution & 0xFF) as u8;
                
                eprintln!("Setting RGB565 mode with resolution {}x{}", width, height);
                
                // Check for data section conflicts
                if self.check_data_section_conflict(new_mode) {
                    eprintln!("WARNING: Potential memory conflict with global data!");
                }
                
                if width > 0 && height > 0 {
                    // Use existing display if available, or create new one
                    if self.rgb565_display.is_none() {
                        self.rgb565_display = Some(RGB565Display::new());
                    }
                    
                    if let Some(ref mut display) = self.rgb565_display {
                        if let Err(e) = display.init(width, height, self.bank_size as usize) {
                            eprintln!("Failed to initialize RGB565 display: {}", e);
                            // Reset to OFF mode on failure
                            self.display_mode = DISP_OFF;
                            return;
                        }
                        eprintln!("RGB565 display initialized successfully: {}x{}", width, height);
                    }
                } else {
                    eprintln!("Invalid display resolution: {}x{}", width, height);
                    self.display_mode = DISP_OFF;
                }
            },
            _ => {}
        }
    }
    
    /// Flush RGB565 display (swap buffers)
    pub(super) fn flush_rgb565_display(&mut self) {
        if let Some(ref mut display) = self.rgb565_display {
            display.flush();
            if self.verbose {
                eprintln!("RGB565: Flushed display");
            }
        } else {
            eprintln!("Warning: flush_rgb565_display called but no display initialized");
        }
    }
    
    /// Render TEXT40 display to terminal
    pub(super) fn flush_text40_display(&mut self) {
        if self.display_mode != DISP_TEXT40 || !self.display_enabled {
            return; // Display not in TEXT40 mode or not enabled
        }
        
        if self.terminal_raw_mode {
            // Render to actual terminal in TEXT40 mode
            let mut stderr = io::stderr();
            
            // Move to top-left
            let _ = stderr.execute(cursor::MoveTo(0, 0));
            
            // Render the 40x25 display
            for row in 0..25 {
                for col in 0..40 {
                    let addr = TEXT40_BASE_WORD + row * 40 + col;
                    if addr < self.memory.len() {
                        let cell = self.memory[addr];
                        let ch = (cell & 0xFF) as u8;
                        let attr = ((cell >> 8) & 0xFF) as u8;
                        
                        // Extract foreground and background colors from attribute byte
                        // Lower 4 bits: foreground color (0-15)
                        // Upper 4 bits: background color (0-15)
                        let fg_color = (attr & 0x0F) as usize;
                        let bg_color = ((attr >> 4) & 0x0F) as usize;
                        
                        // Convert theme color index to terminal Color
                        let theme_to_terminal = |color_idx: usize| -> Color {
                            if color_idx < THEME_COLORS.len() {
                                let (r, g, b) = THEME_COLORS[color_idx];
                                Color::Rgb { r, g, b }
                            } else {
                                // Default to light gray for invalid indices
                                let (r, g, b) = THEME_COLORS[6];
                                Color::Rgb { r, g, b }
                            }
                        };
                        
                        // Set colors
                        let _ = stderr.execute(SetForegroundColor(theme_to_terminal(fg_color)));
                        let _ = stderr.execute(SetBackgroundColor(theme_to_terminal(bg_color)));
                        
                        // Print character
                        if (32..127).contains(&ch) {
                            let _ = stderr.write(&[ch]);
                        } else if ch == 0 {
                            let _ = stderr.write(b" ");
                        } else {
                            let _ = stderr.write(b".");
                        }
                        
                        // Reset colors after each character to prevent color bleeding
                        let _ = stderr.execute(ResetColor);
                    } else {
                        let _ = stderr.write(b" ");
                    }
                }
                // Move to next line
                if row < 24 {
                    let _ = stderr.write(b"\r\n");
                }
            }
            
            let _ = stderr.flush();
        } else if self.verbose {
            // Fallback to debug output
            eprintln!("\n=== TEXT40 Display (40x25) ===");
            for row in 0..25 {
                eprint!("  ");
                for col in 0..40 {
                    let addr = TEXT40_BASE_WORD + row * 40 + col;
                    if addr < self.memory.len() {
                        let cell = self.memory[addr];
                        let ch = (cell & 0xFF) as u8;
                        // Print ASCII character or '.' for non-printable
                        if (32..127).contains(&ch) {
                            eprint!("{}", ch as char);
                        } else if ch == 0 {
                            eprint!(" ");
                        } else {
                            eprint!(".");
                        }
                    }
                }
                eprintln!();
            }
            eprintln!("==============================\n");
        }
    }
}