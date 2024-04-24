use inputbot::KeybdKey::*;
use inputbot::MouseButton::*;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;
const THRESHOLD: f32 = 4.0;
const N_EXTRA_CLICKS: usize = 5; // Number of extra clicks to perform when over threshold
const EXTRA_CLICK_DELAY: Duration = Duration::from_millis(10); // Delay between extra clicks
fn main() {
    let clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let state = Arc::new(AtomicBool::new(true)); // True means clicking is enabled
    let kill_switch = Arc::new(AtomicBool::new(false)); // For stopping the program
    let self_click = Arc::new(AtomicBool::new(false)); // Guard against counting self-generated clicks
    {
        let clicks = clicks.clone();
        let state = state.clone();
        let self_click = self_click.clone();
        thread::spawn(move || {
            LeftButton.bind(move || {
                if !self_click.load(Ordering::Relaxed) {
                    let now = Instant::now();
                    let mut clicks_guard = clicks.lock().unwrap();
                    clicks_guard.push_back(now);
                    if clicks_guard.len() as f32 > THRESHOLD {
                        // Perform extra clicks if the threshold is exceeded
                        perform_extra_clicks(state.clone(), self_click.clone(), N_EXTRA_CLICKS);
                    }
                    while LeftButton.is_pressed() {
                        thread::sleep(Duration::from_millis(1));
                    }
                    // Clean up old clicks
                    while clicks_guard
                        .front()
                        .map_or(false, |t| t.elapsed() > Duration::from_secs(1))
                    {
                        clicks_guard.pop_front();
                    }
                }
            });
            inputbot::handle_input_events();
        });
    }
    {
        let state = state.clone();
        let kill_switch = kill_switch.clone();
        EscapeKey.bind(move || {
            if kill_switch.load(Ordering::Relaxed) {
                // Resetting the kill switch to allow restarting
                kill_switch.store(false, Ordering::Relaxed);
                state.store(true, Ordering::Relaxed);
                println!("Clicking re-enabled.");
            } else {
                kill_switch.store(true, Ordering::Relaxed);
                state.store(false, Ordering::Relaxed);
                println!("Clicking disabled.");
            }
        });
    }
    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
        if kill_switch.load(Ordering::Relaxed) {
            println!("Kill switch activated. Exiting...");
            break;
        }
    }
}
fn perform_extra_clicks(state: Arc<AtomicBool>, self_click: Arc<AtomicBool>, n: usize) {
    self_click.store(true, Ordering::Relaxed); // Set self-click flag to prevent counting these clicks
    for _ in 0..n {
        if state.load(Ordering::Relaxed) {
            LeftButton.release();
            thread::sleep(Duration::from_millis(1));
        }
        LeftButton.press();
        thread::sleep(Duration::from_millis(1));
        LeftButton.release();
        thread::sleep(EXTRA_CLICK_DELAY);
    }
    self_click.store(false, Ordering::Relaxed); // Clear self-click flag after performing clicks
}
