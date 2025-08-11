use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Span, Style, Modifier};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::tui_debugger::{FocusedPane, TuiDebugger};
use crate::vm::VM;

impl TuiDebugger {
    pub(crate) fn draw_breakpoints(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut text = Vec::new();
        
        if self.breakpoints.is_empty() {
            text.push(Line::from(Span::styled(
                "No breakpoints set",
                Style::default().fg(Color::DarkGray)
            )));
            text.push(Line::from(""));
            text.push(Line::from(Span::raw("Press 'b' at cursor")));
            text.push(Line::from(Span::raw("or Shift+B for address")));
        } else {
            // Sort breakpoints for consistent display
            let mut sorted_breakpoints: Vec<_> = self.breakpoints.iter().collect();
            sorted_breakpoints.sort();
            
            for &addr in sorted_breakpoints.iter() {
                let mut spans = vec![];
                
                // Mark current PC breakpoint
                let pc = vm.registers[ripple_asm::Register::Pc as usize] as usize;
                let pcb = vm.registers[ripple_asm::Register::Pcb as usize] as usize;
                let current_addr = pcb * vm.bank_size as usize + pc;
                
                if *addr == current_addr {
                    spans.push(Span::styled("► ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                } else {
                    spans.push(Span::raw("  "));
                }
                
                // Show breakpoint address
                spans.push(Span::styled(
                    format!("{:04X}", addr),
                    Style::default().fg(Color::Red)
                ));
                
                // Show instruction at that address if available
                if *addr < vm.instructions.len() {
                    let instr = &vm.instructions[*addr];
                    let instr_str = self.format_instruction(instr);
                    spans.push(Span::raw(": "));
                    spans.push(Span::styled(
                        instr_str,
                        Style::default().fg(Color::White)
                    ));
                }
                
                text.push(Line::from(spans));
            }
        }
        
        let title = format!(" Breakpoints [{}] ", 
            self.breakpoints.len()
        );
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));
        
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
}