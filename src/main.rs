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
const MULTIPLIER: f32 = 2.0;
const DELAY: Duration = Duration::from_millis(50);
fn main() {
    let clicks = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let clicks_simulated = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    let state = Arc::new(AtomicBool::new(false));
    {
        let clicks = clicks.clone();
        let state = state.clone();
        thread::spawn(move || {
            LeftButton.bind(move || {
                clicks.lock().unwrap().push_back(Instant::now());
                state.store(true, Ordering::Relaxed);
                while LeftButton.is_pressed() {
                    thread::sleep(Duration::from_millis(1));
                }
                state.store(false, Ordering::Relaxed);
            });
            inputbot::handle_input_events();
        });
    }
    {
        let clicks = clicks.clone();
        let clicks_simulated = clicks_simulated.clone();
        thread::spawn(move || loop {
            thread::sleep(DELAY);
            let mut mg_clicks = clicks.lock().unwrap();
            let mut mg_clicks_simulated = clicks_simulated.lock().unwrap();
            while mg_clicks.len() > 0 && mg_clicks[0].elapsed() > Duration::from_secs(1) {
                mg_clicks.pop_front();
            }
            while mg_clicks_simulated.len() > 0
                && mg_clicks_simulated[0].elapsed() > Duration::from_secs(1)
            {
                mg_clicks_simulated.pop_front();
            }
            let cps = mg_clicks.len();
            let cps_simulated = mg_clicks_simulated.len();
            println!(
                "CPS: {}, SIM: {}, {:.0?}",
                cps,
                cps_simulated,
                mg_clicks.iter().map(|x| x.elapsed()).collect::<Vec<_>>()
            );
        });
    }
    loop {
        let cps = (clicks.lock().unwrap().len() + 1) as f32;
        let millis = (1000.0 / (cps * MULTIPLIER)) as u64;
        thread::sleep(Duration::from_millis(millis));
        let mg_clicks = clicks.lock().unwrap();
        if let Some(latest) = mg_clicks.back() {
            if cps > THRESHOLD && latest.elapsed() < DELAY {
                drop(mg_clicks);
                click(state.clone());
                clicks_simulated.lock().unwrap().push_back(Instant::now());
                clicks.lock().unwrap().pop_back();
            }
        }
    }
}
fn click(state: Arc<AtomicBool>) {
    thread::spawn(move || {
        if state.load(Ordering::Relaxed) {
            LeftButton.release();
            thread::sleep(Duration::from_millis(1));
        }
        LeftButton.press();
        thread::sleep(Duration::from_millis(1));
        LeftButton.release();
    });
}
