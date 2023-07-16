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
const MULTIPLIER: f32 = 4.0;
const DELAY: Duration = Duration::from_millis(100);
fn main() {
    let clicks = Arc::new(Mutex::new(VecDeque::new()));
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
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(1));
            let mut clicks = clicks.lock().unwrap();
            while clicks.len() > 0 && clicks[0].elapsed() > Duration::from_secs(1) {
                clicks.pop_front();
            }
            let cps = clicks.len();
            println!(
                "CPS: {}, {:.0?}",
                cps,
                clicks.iter().map(|x| x.elapsed()).collect::<Vec<_>>()
            );
        });
    }
    loop {
        let cps = (clicks.lock().unwrap().len() + 1) as f32;
        let millis = (1000.0 / (cps * MULTIPLIER)) as u64;
        thread::sleep(Duration::from_millis(millis));
        let mg_clicks = clicks.lock().unwrap();
        println!(
            "CPS: {}, {:.0?}",
            cps,
            mg_clicks.iter().map(|x| x.elapsed()).collect::<Vec<_>>()
        );
        if let Some(latest) = mg_clicks.back() {
            if cps > THRESHOLD && latest.elapsed() < DELAY {
                drop(mg_clicks);
                click();
                clicks.lock().unwrap().pop_back();
            }
        }
    }
}
fn click() {
    simulate(&ButtonRelease(Left)).unwrap();
    thread::sleep(Duration::from_millis(1));
    simulate(&ButtonPress(Left)).unwrap();
    thread::sleep(Duration::from_millis(1));
    simulate(&ButtonRelease(Left)).unwrap();
}
