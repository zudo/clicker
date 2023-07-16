use rdev::listen;
use rdev::simulate;
use rdev::Button::Left;
use rdev::Event;
use rdev::EventType::ButtonPress;
use rdev::EventType::ButtonRelease;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;
const THRESHOLD: f32 = 5.0;
const MULTIPLIER: f32 = 2.0;
const DELAY: Duration = Duration::from_millis(50);
fn main() {
    let clicks = Arc::new(Mutex::new(VecDeque::new()));
    let clicks_simulated = Arc::new(Mutex::new(VecDeque::<Instant>::new()));
    {
        let clicks = clicks.clone();
        thread::spawn(move || {
            let callback = move |event: Event| match event.event_type {
                ButtonPress(Left) => {
                    clicks.lock().unwrap().push_back(Instant::now());
                }
                ButtonRelease(Left) => {}
                _ => {}
            };
            if let Err(e) = listen(callback) {
                println!("{:?}", e);
            }
        });
    }
    {
        let clicks = clicks.clone();
        let clicks_simulated = clicks_simulated.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(20));
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
                click();
                clicks_simulated.lock().unwrap().push_back(Instant::now());
                clicks.lock().unwrap().pop_back();
            }
        }
    }
}
fn click() {
    thread::spawn(|| {
        simulate(&ButtonRelease(Left)).unwrap();
        thread::sleep(Duration::from_millis(25));
        simulate(&ButtonPress(Left)).unwrap();
        thread::sleep(Duration::from_millis(25));
        simulate(&ButtonRelease(Left)).unwrap();
    });
}
