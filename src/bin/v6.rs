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
const THRESHOLD: usize = 3;
const THRESHOLD_TIME: Duration = Duration::from_millis(500); // only clicks within this time are considered
const CPS_TARGET: usize = 19;
fn main() {
    let clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let simulated_clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let state = Arc::new(AtomicBool::new(true));
    let self_click = Arc::new(AtomicBool::new(false));
    bind_mouse_clicks(
        clicks.clone(),
        simulated_clicks.clone(),
        state.clone(),
        self_click.clone(),
    );
    bind_keyboard_commands(state.clone());
    spawn_cleanup_thread(clicks.clone(), simulated_clicks.clone());
    spawn_debug_thread(clicks.clone(), simulated_clicks.clone());
    thread::park(); // keep the main thread alive
}
fn bind_mouse_clicks(
    clicks: Arc<Mutex<VecDeque<Instant>>>,
    simulated_clicks: Arc<Mutex<VecDeque<Instant>>>,
    state: Arc<AtomicBool>,
    self_click: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        LeftButton.bind(move || {
            if !self_click.load(Ordering::Relaxed) {
                if !state.load(Ordering::Relaxed) {
                    return;
                }
                println!("{}", "[click]".green());
                let now = Instant::now();
                let mut clicks_guard = clicks.lock().unwrap();
                clicks_guard.push_back(now);
                if clicks_guard.len() > 1 {
                    // Calculate minimum interval from the last two recorded clicks
                    let intervals: Vec<_> = clicks_guard
                        .iter()
                        .zip(clicks_guard.iter().skip(1))
                        .map(|(&prev, &curr)| curr.duration_since(prev))
                        .collect();
                    let min_interval = intervals
                        .iter()
                        .min()
                        .cloned()
                        .unwrap_or(Duration::from_millis(10)); // use a low default value
                    if clicks_guard
                        .iter()
                        .filter(|&t| now.duration_since(*t) <= THRESHOLD_TIME)
                        .collect::<Vec<_>>()
                        .len()
                        > THRESHOLD
                    {
                        perform_extra_clicks(
                            self_click.clone(),
                            clicks_guard.len(),
                            simulated_clicks.clone(),
                            min_interval,
                        );
                    }
                }
                while LeftButton.is_pressed() {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
        inputbot::handle_input_events();
    });
}
fn bind_keyboard_commands(state: Arc<AtomicBool>) {
    CapsLockKey.bind(move || {
        let currently_enabled = state.load(Ordering::Relaxed);
        state.store(!state.load(Ordering::Relaxed), Ordering::Relaxed);
        println!(
            "Clicking {}",
            if currently_enabled {
                "disabled"
            } else {
                "re-enabled"
            }
        );
    });
}
fn spawn_cleanup_thread(
    clicks: Arc<Mutex<VecDeque<Instant>>>,
    simulated_clicks: Arc<Mutex<VecDeque<Instant>>>,
) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let now = Instant::now();
        let mut clicks_guard = clicks.lock().unwrap();
        let mut simulated_clicks_guard = simulated_clicks.lock().unwrap();
        clicks_guard.retain(|&t| now.duration_since(t) <= Duration::from_secs(1));
        simulated_clicks_guard.retain(|&t| now.duration_since(t) <= Duration::from_secs(1));
    });
}
fn spawn_debug_thread(
    clicks: Arc<Mutex<VecDeque<Instant>>>,
    simulated_clicks: Arc<Mutex<VecDeque<Instant>>>,
) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        let clicks_guard = clicks.lock().unwrap();
        let simulated_clicks_guard = simulated_clicks.lock().unwrap();
        println!(
            "              {} {:<2} {} {:<2}",
            "CPS".dimmed(),
            clicks_guard.len(),
            "SIM".dimmed(),
            simulated_clicks_guard.len()
        );
    });
}
fn perform_extra_clicks(
    self_click: Arc<AtomicBool>,
    clicks: usize,
    simulated_clicks: Arc<Mutex<VecDeque<Instant>>>,
    min_interval: Duration,
) {
    let diff = CPS_TARGET.saturating_sub(clicks);
    let n_extra_clicks = (diff as f32 / clicks as f32) + 1.0;
    let extra_click_delay = min_interval / (n_extra_clicks + 1.0) as u32; // Calculate dynamic delay based on the minimum interval
    let extra_click_delay = extra_click_delay.min(Duration::from_millis(1000 / CPS_TARGET as u64));
    for _ in 0..n_extra_clicks as usize {
        thread::sleep(extra_click_delay);
        self_click.store(true, Ordering::Relaxed);
        LeftButton.press();
        println!("{} {:.0?}", "[click]".red(), extra_click_delay);
        thread::sleep(Duration::from_millis(1));
        LeftButton.release();
        self_click.store(false, Ordering::Relaxed);
        simulated_clicks.lock().unwrap().push_back(Instant::now());
    }
}
