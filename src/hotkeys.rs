//! Global hotkeys. Registration has to happen on a thread with a message loop,
//! so a small worker thread owns every `RegisterHotKey` call and reports the
//! results back. Presses arrive through a channel and are polled by the UI.

use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PeekMessageW, PostThreadMessageW, TranslateMessage, MSG,
    PM_NOREMOVE, WM_APP, WM_HOTKEY, WM_QUIT,
};

pub const ID_START_STOP: i32 = 0x0ACE;
pub const ID_PLAYBACK: i32 = 0x0AD1;
pub const ID_RECORD: i32 = 0x0AD2;

enum Command {
    Set {
        id: i32,
        vk: u32,
        reply: Sender<bool>,
    },
}

struct Shared {
    queue: Mutex<VecDeque<Command>>,
    events: Sender<i32>,
}

pub struct Hotkeys {
    shared: Arc<Shared>,
    thread_id: u32,
    events: Receiver<i32>,
}

impl Hotkeys {
    pub fn new() -> Self {
        let (event_tx, event_rx) = channel();
        let (ready_tx, ready_rx) = channel();
        let shared = Arc::new(Shared {
            queue: Mutex::new(VecDeque::new()),
            events: event_tx,
        });
        let worker = Arc::clone(&shared);

        std::thread::Builder::new()
            .name("auto-clicker-hotkeys".into())
            .spawn(move || unsafe {
                let mut message = MSG::default();
                let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE);
                let thread_id = windows::Win32::System::Threading::GetCurrentThreadId();
                let _ = ready_tx.send(thread_id);

                while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                    if message.message == WM_HOTKEY {
                        let _ = worker.events.send(message.wParam.0 as i32);
                        continue;
                    }

                    if message.message == WM_APP {
                        let commands: Vec<Command> = {
                            let mut queue = worker.queue.lock().unwrap_or_else(|e| e.into_inner());
                            queue.drain(..).collect()
                        };

                        for command in commands {
                            let Command::Set { id, vk, reply } = command;
                            let _ = UnregisterHotKey(None, id);
                            let ok = if vk == 0 {
                                true
                            } else {
                                RegisterHotKey(None, id, HOT_KEY_MODIFIERS(0), vk).is_ok()
                            };
                            let _ = reply.send(ok);
                        }
                        continue;
                    }

                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            })
            .ok();

        let thread_id = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .unwrap_or(0);

        Self {
            shared,
            thread_id,
            events: event_rx,
        }
    }

    /// Registers `vk` for `id`; `vk == 0` removes the hotkey.
    pub fn set(&self, id: i32, vk: u32) -> bool {
        if self.thread_id == 0 {
            return false;
        }

        let (reply_tx, reply_rx) = channel();
        {
            let mut queue = self.shared.queue.lock().unwrap_or_else(|e| e.into_inner());
            queue.push_back(Command::Set {
                id,
                vk,
                reply: reply_tx,
            });
        }

        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_APP, WPARAM(0), LPARAM(0));
        }

        reply_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or(false)
    }

    /// Non-blocking drain of hotkey ids pressed since the last frame.
    pub fn poll(&self) -> Vec<i32> {
        let mut pressed = Vec::new();
        while let Ok(id) = self.events.try_recv() {
            pressed.push(id);
        }
        pressed
    }
}

impl Drop for Hotkeys {
    fn drop(&mut self) {
        if self.thread_id != 0 {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
    }
}
