use super::usb_util_keyboard_report;

pub struct KeyboardReport<const N: usize> {
    modifier: u8,
    keycode: [u8; N],
}

impl<const N: usize> KeyboardReport<N> {
    pub fn new() -> Self {
        Self {
            modifier: 0,
            keycode: [0; N],
        }
    }

    pub fn press(&mut self, key: u8) {
        match self.get_modifier(key) {
            Some(modifier) => self.modifier |= modifier,
            None => {
                // Don't add the same key twice
                for i in 0..N {
                    if self.keycode[i] == key {
                        return;
                    }
                }

                let mut found = false;
                for i in 0..N {
                    if self.keycode[i] == 0 {
                        self.keycode[i] = key;
                        found = true;
                        break;
                    }
                }
                if !found {
                    // roll over the first key
                    for i in 1..N {
                        self.keycode.swap(i - 1, i);
                    }
                    self.keycode[N - 1] = key;
                }
            }
        }
        self.send();
    }

    pub fn release(&mut self, key: u8) {
        match self.get_modifier(key) {
            Some(modifier) => self.modifier &= !modifier,
            None => {
                for i in 0..N {
                    if self.keycode[i] == key {
                        self.keycode[i] = 0;
                        break;
                    }
                }
                // Compact the keycode array
                let mut pos = 0;
                for i in 0..N {
                    if self.keycode[i] != 0 {
                        self.keycode.swap(i, pos);
                        pos += 1;
                    }
                }
            }
        }
        self.send();
    }

    pub fn clear(&mut self) {
        self.modifier = 0;
        self.keycode = [0; N];
        self.send();
    }

    fn send(&self) {
        unsafe {
            usb_util_keyboard_report(self.modifier, self.keycode.as_ptr());
        }
    }

    fn get_modifier(&self, key: u8) -> Option<u8> {
        match key {
            0xE0 => Some(0x01), // Left Control
            0xE1 => Some(0x02), // Left Shift
            0xE2 => Some(0x04), // Left Alt
            0xE3 => Some(0x08), // Left GUI
            0xE4 => Some(0x10), // Right Control
            0xE5 => Some(0x20), // Right Shift
            0xE6 => Some(0x40), // Right Alt
            0xE7 => Some(0x80), // Right GUI
            _ => None,
        }
    }
}
