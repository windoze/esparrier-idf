use std::{thread, time};

use esp_idf_hal::gpio::{self, PinDriver};
use esp_idf_sys::esp_timer_get_time;

const MASK: u16 = 0b1111000000111111;

const BUTTON_LONG_PRESS_REPEAT_MS: u32 = 1000;
const BUTTON_LONG_PRESS_DURATION_MS: u32 = 500;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Up,
    Down,
    Held,
}

pub trait ButtonCallback {
    fn on_button_event(&mut self, state: ButtonState);
}

struct Debounce {
    inverted: bool,
    history: u16,
    down_time: u32,
    next_long_time: u32,
}

impl Debounce {
    fn update(&mut self, level: bool) {
        self.history = (self.history << 1) | if level { 1 } else { 0 }
    }

    fn rose(&mut self) -> bool {
        if self.history & MASK == 0b0000000000111111 {
            self.history = 0xFFFF;
            true
        } else {
            false
        }
    }

    fn fell(&mut self) -> bool {
        if self.history & MASK == 0b1111000000000000 {
            self.history = 0;
            true
        } else {
            false
        }
    }

    fn button_down(&mut self) -> bool {
        if self.inverted {
            self.fell()
        } else {
            self.rose()
        }
    }

    fn button_up(&mut self) -> bool {
        if self.inverted {
            self.rose()
        } else {
            self.fell()
        }
    }
}

fn millis() -> u32 {
    (unsafe { esp_timer_get_time() } / 1000) as u32
}

fn button_task<F: ButtonCallback>(pin: gpio::AnyInputPin, mut call_back: F) {
    let driver = PinDriver::input(pin).unwrap();
    let mut button = Debounce {
        inverted: false,
        history: 0,
        down_time: 0,
        next_long_time: 0,
    };
    loop {
        button.update(driver.is_high());
        if button.button_up() {
            button.down_time = 0;
            call_back.on_button_event(ButtonState::Up);
        } else if (button.down_time > 0) && (millis() > button.next_long_time) {
            button.next_long_time += BUTTON_LONG_PRESS_REPEAT_MS;
            call_back.on_button_event(ButtonState::Held);
        } else if button.button_down() && (button.down_time == 0) {
            button.down_time = millis();
            button.next_long_time += BUTTON_LONG_PRESS_DURATION_MS;
            call_back.on_button_event(ButtonState::Down)
        }
        thread::sleep(time::Duration::from_millis(10));
    }
}

pub fn start_button_task<F: ButtonCallback + Send + 'static>(pin: gpio::AnyInputPin, call_back: F) {
    let _ = thread::spawn(move || {
        button_task(pin, call_back)
    });
}