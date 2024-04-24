use inputbot::KeybdKey::*;
use inputbot::MouseButton::*;
use owo_colors::OwoColorize;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;
const THRESHOLD: f32 = 4.0;
const N_EXTRA_CLICKS: usize = 1; // Number of extra clicks to perform when over threshold
const EXTRA_CLICK_DELAY: Duration = Duration::from_millis(20); // Delay between extra clicks
fn main() {
    let clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let simulated_clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let state = Arc::new(AtomicBool::new(true)); // True means clicking is enabled
    let kill_switch = Arc::new(AtomicBool::new(false)); // For stopping the program
    let self_click = Arc::new(AtomicBool::new(false)); // Guard against counting self-generated clicks
    {
        let clicks = clicks.clone();
        let simulated_clicks = simulated_clicks.clone();
        let state = state.clone();
        let self_click = self_click.clone();
        thread::spawn(move || {
            LeftButton.bind(move || {
                if !self_click.load(Ordering::Relaxed) {
                    println!("{}", "[click]".green());
                    let now = Instant::now();
                    let mut clicks_guard = clicks.lock().unwrap();
                    clicks_guard.push_back(now);
                    if clicks_guard.len() as f32 > THRESHOLD {
                        // Perform extra clicks if the threshold is exceeded
                        perform_extra_clicks(
                            state.clone(),
                            self_click.clone(),
                            simulated_clicks.clone(),
                            N_EXTRA_CLICKS,
                        );
                    }
                    while LeftButton.is_pressed() {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            });
            inputbot::handle_input_events();
        });
    }
    {
        let state = state.clone();
        let kill_switch = kill_switch.clone();
        F1Key.bind(move || {
            if kill_switch.load(Ordering::Relaxed) {
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
    // Thread to clean up old clicks every second
    let clicks_cleanup = clicks.clone();
    let simulated_clicks_cleanup = simulated_clicks.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let mut clicks_guard = clicks_cleanup.lock().unwrap();
        let mut simulated_clicks_guard = simulated_clicks_cleanup.lock().unwrap();
        let now = Instant::now();
        clicks_guard.retain(|&t| now.duration_since(t) <= Duration::from_secs(1));
        simulated_clicks_guard.retain(|&t| now.duration_since(t) <= Duration::from_secs(1));
    });
    // Debug output thread for both actual and simulated clicks
    let clicks_debug = clicks.clone();
    let simulated_clicks_debug = simulated_clicks.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        let clicks_guard = clicks_debug.lock().unwrap();
        let simulated_clicks_guard = simulated_clicks_debug.lock().unwrap();
        println!(
            "Actual CPS: {}, Simulated CPS: {}",
            clicks_guard.len(),
            simulated_clicks_guard.len(),
        );
    });
    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
        if kill_switch.load(Ordering::Relaxed) {
            println!("Kill switch activated. Exiting...");
            break;
        }
    }
}
fn perform_extra_clicks(
    state: Arc<AtomicBool>,
    self_click: Arc<AtomicBool>,
    simulated_clicks: Arc<Mutex<VecDeque<Instant>>>,
    n: usize,
) {
    self_click.store(true, Ordering::Relaxed); // Set self-click flag to prevent counting these clicks
    for _ in 0..n {
        if state.load(Ordering::Relaxed) {
            LeftButton.release();
            thread::sleep(Duration::from_millis(1));
        }
        LeftButton.press();
        println!("{}", "[click]".red());
        thread::sleep(Duration::from_millis(1));
        LeftButton.release();
        simulated_clicks.lock().unwrap().push_back(Instant::now()); // Log simulated click
        thread::sleep(EXTRA_CLICK_DELAY);
    }
    self_click.store(false, Ordering::Relaxed); // Clear self-click flag after performing clicks
}
