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
const N_EXTRA_CLICKS: usize = 1;
const EXTRA_CLICK_DELAY: Duration = Duration::from_millis(20);
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
                if clicks_guard.len() as f32 > THRESHOLD {
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
    n: usize,
) {
    self_click.store(true, Ordering::Relaxed);
    for _ in 0..n {
        if state.load(Ordering::Relaxed) {
            LeftButton.release();
            thread::sleep(Duration::from_millis(1));
        }
        LeftButton.press();
        println!("{}", "[click]".red());
        thread::sleep(Duration::from_millis(1));
        LeftButton.release();
        simulated_clicks.lock().unwrap().push_back(Instant::now());
        thread::sleep(EXTRA_CLICK_DELAY);
    }
    self_click.store(false, Ordering::Relaxed);
}
