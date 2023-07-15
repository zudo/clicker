use rdev::listen;
use rdev::simulate;
use rdev::Button::Left;
use rdev::Event;
use rdev::EventType::ButtonPress;
use rdev::EventType::ButtonRelease;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;
const THRESHOLD: f32 = 5.0;
const MULTIPLIER: f32 = 2.0;
const DELAY: u64 = 150;
fn main() {
    let button_press_left = Arc::new(Mutex::new(0));
    let button_press_left_simulated = Arc::new(Mutex::new(0));
    let elapsed = Arc::new(Mutex::new(Duration::from_secs(0)));
    let cps = Arc::new(Mutex::new(0.0));
    {
        let button_press_left = button_press_left.clone();
        let elapsed = elapsed.clone();
        thread::spawn(move || {
            let mut start = Instant::now();
            let callback = move |event: Event| match event.event_type {
                ButtonPress(Left) => {
                    *button_press_left.lock().unwrap() += 1;
                    start = Instant::now();
                }
                ButtonRelease(Left) => {
                    *elapsed.lock().unwrap() = start.elapsed();
                }
                _ => {}
            };
            if let Err(e) = listen(callback) {
                println!("{:?}", e);
            }
        });
    }
    {
        let button_press_left_simulated = button_press_left_simulated.clone();
        let cps = cps.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(DELAY));
            let mut cps = cps.lock().unwrap();
            *cps += *button_press_left.lock().unwrap() as f32 * (1000 / DELAY) as f32;
            let cps_simulated =
                *button_press_left_simulated.lock().unwrap() as f32 * (1000 / DELAY) as f32;
            *cps -= cps_simulated;
            *cps /= 2.0;
            *button_press_left.lock().unwrap() = 0;
            *button_press_left_simulated.lock().unwrap() = 0;
            println!("CPS: {:.2}, SIMULATED: {:.2}", cps, cps_simulated);
        });
    }
    loop {
        thread::sleep(Duration::from_millis(10));
        let cps = *cps.lock().unwrap();
        if cps > THRESHOLD {
            let button_press_left_simulated = button_press_left_simulated.clone();
            thread::spawn(move || {
                *button_press_left_simulated.lock().unwrap() += 1;
                thread::sleep(Duration::from_millis(10));
                simulate(&ButtonRelease(Left)).unwrap();
                thread::sleep(Duration::from_millis(10));
                simulate(&ButtonPress(Left)).unwrap();
                thread::sleep(Duration::from_millis(10));
                simulate(&ButtonRelease(Left)).unwrap();
            });
            thread::sleep(Duration::from_millis(
                (1000.0 / (MULTIPLIER * cps - cps)) as u64,
            ));
        }
    }
}
