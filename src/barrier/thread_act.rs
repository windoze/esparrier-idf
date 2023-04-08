use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::mpsc::{sync_channel, SyncSender},
    thread,
};

use super::Actuator;

pub enum ActMsg {
    Connected,
    Disconnected,
    SetCursorPosition {
        x: u16,
        y: u16,
    },
    MoveCursor {
        x: i16,
        y: i16,
    },
    MouseDown {
        button: i8,
    },
    MouseUp {
        button: i8,
    },
    MouseWheel {
        x: i16,
        y: i16,
    },
    KeyDown {
        key: u16,
        mask: u16,
        button: u16,
    },
    KeyRepeat {
        key: u16,
        mask: u16,
        button: u16,
        count: u16,
    },
    KeyUp {
        key: u16,
        mask: u16,
        button: u16,
    },
    SetClipboard {
        data: Vec<u8>,
    },
    SetOptions {
        opts: HashMap<String, u32>,
    },
    ResetOptions,
    Enter,
    Leave,
    HidKeyDown {
        key: u8,
    },
    HidKeyUp {
        key: u8,
    }
}

pub struct ThreadedActuator<T> {
    screen_width: u16,
    screen_height: u16,
    cursor_x: u16,
    cursor_y: u16,
    tx: SyncSender<ActMsg>,
    _t: PhantomData<T>,
}

impl<T: Actuator + Send + 'static> ThreadedActuator<T> {
    pub fn new(screen_width: u16, screen_height: u16, mut actuator: T) -> Self {
        let (tx, rx) = sync_channel(16);
        let builder = thread::Builder::new().stack_size(16384);
        builder
            .spawn(move || {
                while let Ok(msg) = rx.recv() {
                    match msg {
                        ActMsg::Connected => actuator.connected(),
                        ActMsg::Disconnected => actuator.disconnected(),
                        ActMsg::SetCursorPosition { x, y } => actuator.set_cursor_position(x, y),
                        ActMsg::MoveCursor { x, y } => actuator.move_cursor(x, y),
                        ActMsg::MouseDown { button } => actuator.mouse_down(button),
                        ActMsg::MouseUp { button } => actuator.mouse_up(button),
                        ActMsg::MouseWheel { x, y } => actuator.mouse_wheel(x, y),
                        ActMsg::KeyDown { key, mask, button } => {
                            actuator.key_down(key, mask, button)
                        }
                        ActMsg::KeyRepeat {
                            key,
                            mask,
                            button,
                            count,
                        } => actuator.key_repeat(key, mask, button, count),
                        ActMsg::KeyUp { key, mask, button } => actuator.key_up(key, mask, button),
                        ActMsg::SetClipboard { data } => actuator.set_clipboard(data),
                        ActMsg::SetOptions { opts } => actuator.set_options(opts),
                        ActMsg::ResetOptions => actuator.reset_options(),
                        ActMsg::Enter => actuator.enter(),
                        ActMsg::Leave => actuator.leave(),
                        ActMsg::HidKeyDown { key } => actuator.hid_key_down(key),
                        ActMsg::HidKeyUp { key } => actuator.hid_key_up(key),
                    }
                }
            })
            .expect("Failed to create actuator thread");

        Self {
            screen_width,
            screen_height,
            cursor_x: 0,
            cursor_y: 0,
            tx,
            _t: PhantomData::default(),
        }
    }

    pub fn get_sender(&self) -> SyncSender<ActMsg> {
        self.tx.clone()
    }

    fn send(&self, msg: ActMsg) {
        self.tx.send(msg).unwrap()
    }
}

impl<T: Actuator + Send + 'static> Actuator for ThreadedActuator<T> {
    fn connected(&mut self) {
        self.send(ActMsg::Connected)
    }

    fn disconnected(&mut self) {
        self.send(ActMsg::Disconnected)
    }

    fn get_screen_size(&self) -> (u16, u16) {
        (self.screen_width, self.screen_height)
    }

    fn get_cursor_position(&self) -> (u16, u16) {
        (self.cursor_x, self.cursor_y)
    }

    fn set_cursor_position(&mut self, x: u16, y: u16) {
        self.send(ActMsg::SetCursorPosition { x, y });
        self.cursor_x = x;
        self.cursor_y = y;
    }

    fn move_cursor(&mut self, x: i16, y: i16) {
        self.send(ActMsg::MoveCursor { x, y });
        self.cursor_x = self.cursor_x.wrapping_add_signed(x);
        self.cursor_y = self.cursor_y.wrapping_add_signed(y);
    }

    fn mouse_down(&mut self, button: i8) {
        self.send(ActMsg::MouseDown { button })
    }

    fn mouse_up(&mut self, button: i8) {
        self.send(ActMsg::MouseUp { button })
    }

    fn mouse_wheel(&mut self, x: i16, y: i16) {
        self.send(ActMsg::MouseWheel { x, y })
    }

    fn key_down(&mut self, key: u16, mask: u16, button: u16) {
        self.send(ActMsg::KeyDown { key, mask, button })
    }

    fn key_repeat(&mut self, key: u16, mask: u16, button: u16, count: u16) {
        self.send(ActMsg::KeyRepeat {
            key,
            mask,
            button,
            count,
        })
    }

    fn key_up(&mut self, key: u16, mask: u16, button: u16) {
        self.send(ActMsg::KeyUp { key, mask, button })
    }

    fn set_clipboard(&mut self, data: Vec<u8>) {
        self.send(ActMsg::SetClipboard { data })
    }

    fn set_options(&mut self, opts: HashMap<String, u32>) {
        self.send(ActMsg::SetOptions { opts })
    }

    fn reset_options(&mut self) {
        self.send(ActMsg::ResetOptions)
    }

    fn enter(&mut self) {
        self.send(ActMsg::Enter)
    }

    fn leave(&mut self) {
        self.send(ActMsg::Leave)
    }

    fn hid_key_down(&mut self, key: u8) {
        self.send(ActMsg::HidKeyDown { key })
    }

    fn hid_key_up(&mut self, key: u8) {
        self.send(ActMsg::HidKeyUp { key })
    }
}
