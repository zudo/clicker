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
const THRESHOLD: f32 = 5.0;
const N_EXTRA_CLICKS: usize = 4;
fn main() {
    let clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let simulated_clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let state = Arc::new(AtomicBool::new(true));
    let kill_switch = Arc::new(AtomicBool::new(false));
    let self_click = Arc::new(AtomicBool::new(false));
    bind_mouse_clicks(
        clicks.clone(),
        simulated_clicks.clone(),
        state.clone(),
        self_click.clone(),
    );
    bind_keyboard_commands(state.clone(), kill_switch.clone());
    spawn_cleanup_thread(clicks.clone(), simulated_clicks.clone());
    spawn_debug_thread(clicks.clone(), simulated_clicks.clone());
    main_loop(kill_switch.clone());
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
                    if clicks_guard.len() as f32 > THRESHOLD {
                        perform_extra_clicks(
                            state.clone(),
                            self_click.clone(),
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
fn bind_keyboard_commands(state: Arc<AtomicBool>, kill_switch: Arc<AtomicBool>) {
    F1Key.bind(move || {
        let currently_enabled =
            kill_switch.swap(!kill_switch.load(Ordering::Relaxed), Ordering::Relaxed);
        state.store(!currently_enabled, Ordering::Relaxed);
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
            "Actual CPS: {}, Simulated CPS: {}",
            clicks_guard.len(),
            simulated_clicks_guard.len()
        );
    });
}
fn main_loop(kill_switch: Arc<AtomicBool>) {
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
    min_interval: Duration,
) {
    self_click.store(true, Ordering::Relaxed);
    let extra_click_delay = min_interval / (N_EXTRA_CLICKS as u32 + 1); // Calculate dynamic delay based on the minimum interval
    for _ in 0..N_EXTRA_CLICKS {
        if state.load(Ordering::Relaxed) {
            LeftButton.release();
            thread::sleep(extra_click_delay);
        }
        LeftButton.press();
        println!("{} {:.0?}", "[click]".red(), extra_click_delay);
        thread::sleep(Duration::from_millis(1));
        LeftButton.release();
        simulated_clicks.lock().unwrap().push_back(Instant::now());
        thread::sleep(extra_click_delay);
    }
    self_click.store(false, Ordering::Relaxed);
}
