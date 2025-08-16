use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers, MouseEventKind};
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

pub enum Event<I> {
    Input(I),
    Mouse(MouseEvent),
    Tick,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
}

pub struct EventHandler {
    pub rx: mpsc::Receiver<Event<KeyEvent>>,
    _tx: mpsc::Sender<Event<KeyEvent>>,
    paused: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();
        let paused = Arc::new(AtomicBool::new(false));
        let paused_clone = paused.clone();
        
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                // If paused, just sleep and continue
                if paused_clone.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                    last_tick = Instant::now(); // Reset tick timer
                    continue;
                }
                
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap() {
                    if let CEvent::Key(key) = event::read().unwrap() {
                        let key_event = KeyEvent {
                            code: key.code,
                            modifiers: key.modifiers,
                        };
                        if event_tx.send(Event::Input(key_event)).is_err() {
                            return;
                        }
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if event_tx.send(Event::Tick).is_err() {
                        return;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        EventHandler { rx, _tx: tx, paused }
    }
    
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
        // Give the thread time to pause
        thread::sleep(Duration::from_millis(50));
    }
    
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}