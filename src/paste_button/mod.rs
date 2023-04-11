use std::sync::mpsc::SyncSender;

use esp_idf_hal::gpio;

use crate::{barrier::ActMsg, keycodes::ASCII_2_HID, CLIPBOARD};

mod button;

use button::{ButtonCallback, ButtonState};

pub struct PasteButton {
    tx: SyncSender<ActMsg>,
}

impl PasteButton {
    pub fn new(tx: SyncSender<ActMsg>) -> Self {
        Self { tx }
    }

    /**
     * Only supports 7bit ASCII characters as other keys don't have a USB HID code
     */
    fn send_char(&self, byte: u8) {
        if byte > 0x7F {
            return;
        }
        let [k, m] = ASCII_2_HID[byte as usize];
        if k == 0 {
            return;
        }
        if m != 0 {
            self.tx.send(ActMsg::HidKeyDown { key: m }).ok();
            self.tx.send(ActMsg::HidKeyDown { key: k }).ok();
            self.tx.send(ActMsg::HidKeyUp { key: k }).ok();
            self.tx.send(ActMsg::HidKeyUp { key: m }).ok();
        } else {
            self.tx.send(ActMsg::HidKeyDown { key: k }).ok();
            self.tx.send(ActMsg::HidKeyUp { key: k }).ok();
        }
    }
}

impl ButtonCallback for PasteButton {
    fn on_button_event(&mut self, state: ButtonState) {
        match state {
            ButtonState::Up => {}
            ButtonState::Down => {
                let data = CLIPBOARD.lock().unwrap();
                for c in data.iter() {
                    if *c == 0 {
                        break;
                    }
                    self.send_char(*c);
                }
            }
            ButtonState::Held => {}
        }
    }
}

pub fn start_paste_button_task(pin: gpio::AnyInputPin, tx: SyncSender<ActMsg>) {
    button::start_button_task(pin, PasteButton::new(tx));
}
