use rdev::listen;
use rdev::Button::Left;
use rdev::Event;
use rdev::EventType::ButtonPress;
use rdev::EventType::ButtonRelease;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
const THRESHOLD: usize = 5;
const MULTIPLIER: f32 = 1.5; // VIRTUAL_CPS = CPS * MULTIPLIER
const DELAY: u64 = 1000; // ms
fn main() {
    let counter = Arc::new(Mutex::new(0.0));
    let cps = Arc::new(Mutex::new(0.0));
    {
        let counter = counter.clone();
        thread::spawn(move || {
            let callback = move |event: Event| match event.event_type {
                ButtonPress(Left) => {
                    *counter.lock().unwrap() += 1.0;
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
        let counter = counter.clone();
        let cps = cps.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(DELAY));
            let mut cps = cps.lock().unwrap();
            let mut counter = counter.lock().unwrap();
            *cps += *counter;
            *counter = 0.0;
            if *cps > THRESHOLD as f32 {
                println!("click");
            }
        });
    }
    loop {
        thread::sleep(Duration::from_secs(1));
        let mut cps = cps.lock().unwrap();
        let counter = counter.lock().unwrap();
        let cps_float = *cps as f32 / MULTIPLIER;
        let counter_float = *counter as f32;
        println!("CPS: {:.2}", cps_float);
        println!("Counter: {:.2}", counter_float);
        *cps /= 2.0;
    }
}
