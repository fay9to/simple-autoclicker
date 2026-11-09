//! Recording and playback of mouse clicks and movements.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::hooks::Hook;
use crate::native::{self, MouseButtonKind};

#[derive(Clone, Copy, Debug)]
pub struct RecordedEvent {
    pub is_move: bool,
    pub button: MouseButtonKind,
    pub down: bool,
    pub x: i32,
    pub y: i32,
    pub time_ms: i64,
}

pub struct Recorder {
    pub hook: Hook,
    pub events: Vec<RecordedEvent>,
    pub recording: bool,
    pub playing: bool,
    pub restore_after: bool,
    start: Instant,
    last_move_ms: i64,
    last_move: (i32, i32),
    play_stop: Arc<AtomicBool>,
    play_thread: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            hook: Hook::new(),
            events: Vec::new(),
            recording: false,
            playing: false,
            restore_after: false,
            start: Instant::now(),
            last_move_ms: -1000,
            last_move: (0, 0),
            play_stop: Arc::new(AtomicBool::new(false)),
            play_thread: None,
        }
    }

    pub fn click_count(&self) -> usize {
        self.events.iter().filter(|e| !e.is_move).count() / 2
    }

    pub fn move_count(&self) -> usize {
        self.events.iter().filter(|e| e.is_move).count()
    }

    pub fn start_recording(&mut self) -> bool {
        if self.playing {
            return false;
        }
        if !self.hook.is_active() {
            return false;
        }

        self.events.clear();
        self.last_move_ms = -1000;
        self.start = Instant::now();
        self.recording = true;
        self.hook.set_recording(true);
        true
    }

    pub fn stop_recording(&mut self) {
        if !self.recording {
            return;
        }
        self.recording = false;
        self.hook.set_recording(false);
    }

    /// Drains hook events into the recording. `own_window` filters clicks that
    /// land on our own UI so playing them back cannot press our buttons.
    pub fn pump(&mut self, own_window: Option<(i32, i32, i32, i32)>, minimized: bool) {
        while let Ok(event) = self.hook.events.try_recv() {
            if !self.recording {
                continue;
            }

            if !event.is_move {
                if !minimized {
                    if let Some((left, top, right, bottom)) = own_window {
                        if event.x >= left
                            && event.x < right
                            && event.y >= top
                            && event.y < bottom
                        {
                            continue;
                        }
                    }
                }
            }

            let time_ms = event.at.saturating_duration_since(self.start).as_millis() as i64;

            if event.is_move {
                // At most ~60 points a second and only when the cursor moved.
                if time_ms - self.last_move_ms < 12
                    && (event.x - self.last_move.0).abs() < 3
                    && (event.y - self.last_move.1).abs() < 3
                {
                    continue;
                }
                self.last_move_ms = time_ms;
                self.last_move = (event.x, event.y);
            }

            self.events.push(RecordedEvent {
                is_move: event.is_move,
                button: event.button,
                down: event.down,
                x: event.x,
                y: event.y,
                time_ms,
            });
        }
    }

    pub fn start_playback(&mut self) -> bool {
        if self.recording || self.events.is_empty() {
            return false;
        }

        self.playing = true;
        self.play_stop.store(false, Ordering::SeqCst);
        let events = self.events.clone();
        let stop = Arc::clone(&self.play_stop);

        self.play_thread = std::thread::Builder::new()
            .name("auto-clicker-playback".into())
            .spawn(move || {
                let clock = Instant::now();
                for event in events {
                    while !stop.load(Ordering::SeqCst) {
                        let now = clock.elapsed().as_millis() as i64;
                        let left = event.time_ms - now;
                        if left <= 0 {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis((left as u64).min(20)));
                    }

                    if stop.load(Ordering::SeqCst) {
                        return;
                    }

                    native::move_absolute(event.x, event.y);
                    if !event.is_move {
                        native::send_button(event.button, event.down);
                    }
                }
            })
            .ok();

        true
    }

    pub fn stop_playback(&mut self) {
        self.play_stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.play_thread.take() {
            let _ = thread.join();
        }
        self.playing = false;
    }

    /// Called from the UI thread once the playback thread has finished.
    pub fn poll_playback(&mut self) {
        if self.playing {
            let done = self
                .play_thread
                .as_ref()
                .map(|thread| thread.is_finished())
                .unwrap_or(true);
            if done {
                if let Some(thread) = self.play_thread.take() {
                    let _ = thread.join();
                }
                self.playing = false;
            }
        }
    }
}
