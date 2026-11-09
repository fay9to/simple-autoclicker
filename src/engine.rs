//! The clicking loop. Runs on a worker thread, keeps its own precise clock and
//! reports state through atomics so the UI can poll it without locking.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::native::{self, MouseButtonKind};

#[derive(Clone, Copy, Debug)]
pub struct ClickConfig {
    pub interval_ms: u64,
    pub button: MouseButtonKind,
    pub double_click: bool,
    pub repeat_forever: bool,
    pub repeat_count: u32,
    pub fixed_position: bool,
    pub x: i32,
    pub y: i32,
}

#[derive(Default)]
pub struct Engine {
    running: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    clicks: Arc<AtomicU32>,
    thread: Option<JoinHandle<()>>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn take_finished(&self) -> bool {
        self.finished.swap(false, Ordering::SeqCst)
    }

    pub fn start(&mut self, settings: ClickConfig) {
        if self.is_running() {
            return;
        }

        let running = Arc::clone(&self.running);
        let finished = Arc::clone(&self.finished);
        let clicks = Arc::clone(&self.clicks);

        running.store(true, Ordering::SeqCst);
        finished.store(false, Ordering::SeqCst);
        clicks.store(0, Ordering::SeqCst);

        self.thread = std::thread::Builder::new()
            .name("auto-clicker".into())
            .spawn(move || {
                let interval = settings.interval_ms.max(1);
                let period = Duration::from_millis(interval);
                let mut next = Instant::now();
                let mut count: u32 = 0;
                let limit = settings.repeat_count.max(1);

                while running.load(Ordering::SeqCst) {
                    if settings.fixed_position {
                        native::set_cursor_pos(settings.x, settings.y);
                    }

                    native::click(settings.button, settings.double_click);
                    count = count.wrapping_add(1);
                    clicks.store(count, Ordering::SeqCst);

                    if !settings.repeat_forever && count >= limit {
                        break;
                    }

                    next += period;
                    let now = Instant::now();
                    if next < now {
                        // Never catch up: after a stall a burst of clicks would
                        // be much worse than a slightly longer interval.
                        next = now;
                    }

                    while running.load(Ordering::SeqCst) {
                        let now = Instant::now();
                        if next <= now {
                            break;
                        }
                        let left = next - now;
                        if left > Duration::from_millis(2) {
                            std::thread::sleep(left - Duration::from_millis(1));
                        } else {
                            std::hint::spin_loop();
                        }
                    }
                }

                running.store(false, Ordering::SeqCst);
                finished.store(true, Ordering::SeqCst);
            })
            .ok();
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
