//! Low-level mouse hook used by Record. The hook lives on its own thread with
//! a message loop for the whole lifetime of the app; events are only forwarded
//! while recording is armed.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, MSG, MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_QUIT, WM_RBUTTONDOWN,
    WM_RBUTTONUP,
};

use crate::native::MouseButtonKind;

#[derive(Clone, Copy, Debug)]
pub struct HookEvent {
    pub is_move: bool,
    pub button: MouseButtonKind,
    pub down: bool,
    pub x: i32,
    pub y: i32,
    pub at: Instant,
}

static SENDER: Mutex<Option<Sender<HookEvent>>> = Mutex::new(None);
static RECORDING: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && RECORDING.load(Ordering::Relaxed) {
        let message = wparam.0 as u32;
        let data = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        let mut event: Option<HookEvent> = None;

        if message == WM_MOUSEMOVE {
            event = Some(HookEvent {
                is_move: true,
                button: MouseButtonKind::Left,
                down: false,
                x: data.pt.x,
                y: data.pt.y,
                at: Instant::now(),
            });
        } else {
            let parsed = match message {
                WM_LBUTTONDOWN => Some((MouseButtonKind::Left, true)),
                WM_LBUTTONUP => Some((MouseButtonKind::Left, false)),
                WM_RBUTTONDOWN => Some((MouseButtonKind::Right, true)),
                WM_RBUTTONUP => Some((MouseButtonKind::Right, false)),
                WM_MBUTTONDOWN => Some((MouseButtonKind::Middle, true)),
                WM_MBUTTONUP => Some((MouseButtonKind::Middle, false)),
                _ => None,
            };
            if let Some((button, down)) = parsed {
                event = Some(HookEvent {
                    is_move: false,
                    button,
                    down,
                    x: data.pt.x,
                    y: data.pt.y,
                    at: Instant::now(),
                });
            }
        }

        if let Some(event) = event {
            if let Ok(guard) = SENDER.lock() {
                if let Some(sender) = guard.as_ref() {
                    let _ = sender.send(event);
                }
            }
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Handle for the hook thread plus the channel that carries recorded events.
pub struct Hook {
    pub events: Receiver<HookEvent>,
    /// True when the system accepted the hook.
    pub active: Arc<AtomicBool>,
    thread_id: u32,
}

impl Hook {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let active = Arc::new(AtomicBool::new(false));
        let thread_active = Arc::clone(&active);
        let (ready_tx, ready_rx) = channel();

        std::thread::Builder::new()
            .name("auto-clicker-hook".into())
            .spawn(move || unsafe {
                let mut message = MSG::default();
                // Forces Windows to create the thread message queue.
                let _ = windows::Win32::UI::WindowsAndMessaging::PeekMessageW(
                    &mut message,
                    None,
                    0,
                    0,
                    windows::Win32::UI::WindowsAndMessaging::PM_NOREMOVE,
                );

                let thread_id = windows::Win32::System::Threading::GetCurrentThreadId();
                let module = GetModuleHandleW(None).ok();
                let instance = module.map(|m| HINSTANCE(m.0));
                let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), instance, 0);

                let hook = match hook {
                    Ok(hook) => hook,
                    Err(_) => {
                        let _ = ready_tx.send((thread_id, false));
                        return;
                    }
                };

                if let Ok(mut guard) = SENDER.lock() {
                    *guard = Some(tx);
                }
                thread_active.store(true, Ordering::SeqCst);
                let _ = ready_tx.send((thread_id, true));

                while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }

                let _ = UnhookWindowsHookEx(hook);
                thread_active.store(false, Ordering::SeqCst);
                if let Ok(mut guard) = SENDER.lock() {
                    *guard = None;
                }
            })
            .ok();

        let (thread_id, ok) = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .unwrap_or((0, false));
        active.store(ok, Ordering::SeqCst);

        Self {
            events: rx,
            active,
            thread_id,
        }
    }

    pub fn set_recording(&self, recording: bool) {
        RECORDING.store(recording, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        if self.thread_id != 0 {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
    }
}
