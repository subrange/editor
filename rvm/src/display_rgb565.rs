use std::sync::{Arc, Mutex};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoopBuilder, ControlFlow};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

/// RGB565 color format: RRRRRGGGGGGBBBBB
fn rgb565_to_rgb888(color: u16) -> [u8; 3] {
    let r = ((color >> 11) & 0x1F) as u8;
    let g = ((color >> 5) & 0x3F) as u8;
    let b = (color & 0x1F) as u8;
    
    // Scale to 8-bit values
    let r8 = (r << 3) | (r >> 2);
    let g8 = (g << 2) | (g >> 4);
    let b8 = (b << 3) | (b >> 2);
    
    [r8, g8, b8]
}

/// Shared display state between VM and display
pub struct RGB565State {
    pub width: u8,
    pub height: u8,
    pub initialized: bool,
    pub front_buffer: Vec<u16>,
    pub back_buffer: Vec<u16>,
    pub should_quit: bool,
    // Keyboard state for RGB565 mode
    pub key_up: bool,
    pub key_down: bool,
    pub key_left: bool,
    pub key_right: bool,
    pub key_z: bool,
    pub key_x: bool,
}

impl RGB565State {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            initialized: false,
            front_buffer: Vec::new(),
            back_buffer: Vec::new(),
            should_quit: false,
            key_up: false,
            key_down: false,
            key_left: false,
            key_right: false,
            key_z: false,
            key_x: false,
        }
    }
    
    pub fn init(&mut self, width: u8, height: u8, bank_size: usize) -> Result<(), String> {
        let pixels_needed = width as usize * height as usize;
        let available_space = (bank_size - 32) / 2; // 32 for MMIO, divide by 2 for double buffer
        
        if pixels_needed > available_space {
            return Err(format!(
                "Display resolution {}x{} requires {} words, but only {} available in bank",
                width, height, pixels_needed * 2, available_space * 2
            ));
        }
        
        self.width = width;
        self.height = height;
        self.front_buffer = vec![0; pixels_needed];
        self.back_buffer = vec![0; pixels_needed];
        self.initialized = true;
        
        Ok(())
    }
    
    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
    }
}

/// RGB565 Display handler
pub struct RGB565Display {
    state: Arc<Mutex<RGB565State>>,
}

impl RGB565Display {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RGB565State::new())),
        }
    }
    
    /// Get a clone of the shared state
    pub fn get_state(&self) -> Arc<Mutex<RGB565State>> {
        Arc::clone(&self.state)
    }
    
    /// Initialize display with given resolution
    pub fn init(&mut self, width: u8, height: u8, bank_size: usize) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.init(width, height, bank_size)
    }
    
    /// Write to back buffer at memory address
    pub fn write_memory(&mut self, addr: usize, value: u16) {
        let mut state = self.state.lock().unwrap();
        if !state.initialized {
            return;
        }
        
        // Memory layout: 32 words MMIO, then front buffer, then back buffer
        let buffer_size = state.width as usize * state.height as usize;
        let back_buffer_start = 32 + buffer_size;
        
        if addr >= back_buffer_start && addr < back_buffer_start + buffer_size {
            let pixel_idx = addr - back_buffer_start;
            if pixel_idx < state.back_buffer.len() {
                state.back_buffer[pixel_idx] = value;
                // Debug: log first few pixel writes
                static mut WRITE_COUNT: usize = 0;
                unsafe {
                    if WRITE_COUNT < 10 {
                        eprintln!("RGB565: Write pixel[{}] = 0x{:04x}", pixel_idx, value);
                        WRITE_COUNT += 1;
                    }
                }
            }
        }
    }
    
    /// Read from memory at address
    pub fn read_memory(&self, addr: usize) -> Option<u16> {
        let state = self.state.lock().unwrap();
        if !state.initialized {
            return None;
        }
        
        // Memory layout: 32 words MMIO, then front buffer, then back buffer
        let buffer_size = state.width as usize * state.height as usize;
        let front_buffer_start = 32;
        let back_buffer_start = 32 + buffer_size;
        
        if addr >= front_buffer_start && addr < front_buffer_start + buffer_size {
            let pixel_idx = addr - front_buffer_start;
            if pixel_idx < state.front_buffer.len() {
                return Some(state.front_buffer[pixel_idx]);
            }
        } else if addr >= back_buffer_start && addr < back_buffer_start + buffer_size {
            let pixel_idx = addr - back_buffer_start;
            if pixel_idx < state.back_buffer.len() {
                return Some(state.back_buffer[pixel_idx]);
            }
        }
        
        None
    }
    
    /// Swap buffers (called when HDR_DISP_FLUSH is written)
    pub fn flush(&mut self) {
        let mut state = self.state.lock().unwrap();
        if state.initialized {
            state.swap_buffers();
        }
    }
    
    /// Shutdown display
    pub fn shutdown(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.should_quit = true;
    }
}

/// Run the RGB565 display window on the main thread
/// This function takes over the thread and doesn't return until the window is closed
pub fn run_rgb565_display(state: Arc<Mutex<RGB565State>>) -> Result<(), Box<dyn std::error::Error>> {
    // Start with a default window size
    let default_width = 256u32;
    let default_height = 256u32;
    eprintln!("Opening display window ({}x{})...", default_width, default_height);
    
    // Create event loop and window
    let event_loop = EventLoopBuilder::new().build();
    let mut input = WinitInputHelper::new();
    
    let window = WindowBuilder::new()
        .with_title("Ripple VM - RGB565 Display")
        .with_inner_size(LogicalSize::new(default_width * 2, default_height * 2))
        .with_resizable(false)
        .build(&event_loop)?;
    
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(default_width, default_height, surface_texture)?;
    
    // Track actual display size once initialized
    let mut actual_width = default_width;
    let mut actual_height = default_height;
    let mut display_active = false;
    
    event_loop.run(move |event, _, control_flow| {
        // Check if we should quit
        {
            let s = state.lock().unwrap();
            if s.should_quit {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        
        // Update input helper first to capture keyboard events
        if input.update(&event) {
            // Update keyboard state based on input
            let mut s = state.lock().unwrap();
            
            // Update key states based on what's currently pressed
            s.key_up = input.key_held(winit::event::VirtualKeyCode::Up);
            s.key_down = input.key_held(winit::event::VirtualKeyCode::Down);
            s.key_left = input.key_held(winit::event::VirtualKeyCode::Left);
            s.key_right = input.key_held(winit::event::VirtualKeyCode::Right);
            s.key_z = input.key_held(winit::event::VirtualKeyCode::Z);
            s.key_x = input.key_held(winit::event::VirtualKeyCode::X);
        }
        
        // Handle events
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::RedrawRequested(_) => {
                // Check if display is now initialized and update if needed
                let s = state.lock().unwrap();
                if s.initialized && !display_active {
                    // Display just got initialized!
                    actual_width = s.width as u32;
                    actual_height = s.height as u32;
                    display_active = true;
                    eprintln!("Display activated: {}x{}", actual_width, actual_height);
                    
                    // Recreate pixels buffer with actual size
                    let window_size = window.inner_size();
                    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
                    pixels = Pixels::new(actual_width, actual_height, surface_texture).unwrap();
                }
                
                if display_active && s.initialized {
                    // Copy front buffer to pixels framebuffer
                    let frame = pixels.frame_mut();
                    let buffer_len = (actual_width * actual_height) as usize;
                    for i in 0..buffer_len.min(s.front_buffer.len()) {
                        let pixel = s.front_buffer[i];
                        let [r, g, b] = rgb565_to_rgb888(pixel);
                        let offset = i * 4;
                        if offset + 3 < frame.len() {
                            frame[offset] = r;
                            frame[offset + 1] = g;
                            frame[offset + 2] = b;
                            frame[offset + 3] = 255; // Alpha
                        }
                    }
                } else {
                    // Show a placeholder pattern while waiting
                    let frame = pixels.frame_mut();
                    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                        let x = (i as u32) % default_width;
                        let y = (i as u32) / default_width;
                        let checker = ((x / 16) + (y / 16)) % 2 == 0;
                        let color = if checker { 64 } else { 32 };
                        pixel[0] = color;
                        pixel[1] = color;
                        pixel[2] = color;
                        pixel[3] = 255;
                    }
                }
                drop(s); // Release lock before rendering
                
                let _ = pixels.render();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    })
}

impl Drop for RGB565Display {
    fn drop(&mut self) {
        self.shutdown();
    }
}