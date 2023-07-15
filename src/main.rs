use rdev::listen;
use rdev::Button::Left;
use rdev::Event;
use rdev::EventType::ButtonPress;
use rdev::EventType::ButtonRelease;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
const THRESHOLD: f32 = 5.0;
const DELAY: u64 = 100;
fn main() {
    let counter = Arc::new(Mutex::new(0.0));
    let mut cps = 0.0;
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
    loop {
        thread::sleep(Duration::from_millis(DELAY));
        let mut counter = counter.lock().unwrap();
        cps += *counter * (1000 / DELAY) as f32;
        cps /= 2.0;
        *counter = 0.0;
        println!("CPS: {:.2}", cps);
        if cps > THRESHOLD {
            println!("click");
        }
    }
}
