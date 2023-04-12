pub trait Actuator {
    fn connected(&mut self);

    fn disconnected(&mut self);

    fn get_screen_size(&self) -> (u16, u16);

    fn get_cursor_position(&self) -> (u16, u16);

    fn set_cursor_position(&mut self, x: u16, y: u16);

    fn move_cursor(&mut self, x: i16, y: i16);

    fn mouse_down(&mut self, button: i8);

    fn mouse_up(&mut self, button: i8);

    fn mouse_wheel(&mut self, x: i16, y: i16);

    fn key_down(&mut self, key: u16, mask: u16, button: u16);

    fn key_repeat(&mut self, key: u16, mask: u16, button: u16, count: u16);

    fn key_up(&mut self, key: u16, mask: u16, button: u16);

    fn set_clipboard(&mut self, data: Vec<u8>);

    fn set_options(&mut self, heartbeat: u32);

    fn reset_options(&mut self);

    fn enter(&mut self);

    fn leave(&mut self);

    fn hid_key_down(&mut self, key: u8);

    fn hid_key_up(&mut self, key: u8);
}