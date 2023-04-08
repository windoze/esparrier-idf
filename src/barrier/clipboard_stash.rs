use std::cmp::min;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ClipboardFormat {
    Text = 0,
    Html = 1,
    Bitmap = 2,
}

#[derive(Debug, Default)]
pub enum States {
    #[default]
    Start,
    NumFormat1(u8),
    NumFormat2(u8, u8),
    NumFormat3(u8, u8, u8),
    Format0,
    Format1(u8),
    Format2(u8, u8),
    Format3(u8, u8, u8),
    Size0,
    Size1(u8),
    Size2(u8, u8),
    Size3(u8, u8, u8),
    Data(u32, u32), // max length, current length
    End,
}

const MAX_SIZE: usize = 1024;

/**
 * ClipboardStash keeps only text format with size limit
 */
#[derive(Debug, Default)]
pub struct ClipboardStash {
    pub state: States,
    pub num_format: u32,
    pub current_index: u32,
    data: Vec<(ClipboardFormat, Vec<u8>)>,
}

impl ClipboardStash {
    pub fn feed(&mut self, chunk: &[u8]) {
        for b in chunk {
            self.state = match self.state {
                States::Start => States::NumFormat1(*b),
                States::NumFormat1(b0) => States::NumFormat2(b0, *b),
                States::NumFormat2(b0, b1) => States::NumFormat3(b0, b1, *b),
                States::NumFormat3(b0, b1, b2) => {
                    self.num_format = u32::from_be_bytes([b0, b1, b2, *b]);
                    self.current_index = 0;
                    if self.num_format == 0 {
                        States::End
                    } else {
                        States::Format0
                    }
                }
                States::Format0 => States::Format1(*b),
                States::Format1(b0) => States::Format2(b0, *b),
                States::Format2(b0, b1) => States::Format3(b0, b1, *b),
                States::Format3(b0, b1, b2) => {
                    let format = u32::from_be_bytes([b0, b1, b2, *b]);
                    let format = match format {
                        0 => ClipboardFormat::Text,
                        1 => ClipboardFormat::Html,
                        2 => ClipboardFormat::Bitmap,
                        _ => panic!("Wrong format"),
                    };
                    self.data.push((format, Vec::new()));
                    States::Size0
                }
                States::Size0 => States::Size1(*b),
                States::Size1(b0) => States::Size2(b0, *b),
                States::Size2(b0, b1) => States::Size3(b0, b1, *b),
                States::Size3(b0, b1, b2) => {
                    let length = u32::from_be_bytes([b0, b1, b2, *b]);
                    if self.get_current_format() == ClipboardFormat::Text {
                        self.data.last_mut().unwrap().1.resize(min(MAX_SIZE, length as usize), 0);
                    }
                    States::Data(length, 0)
                }
                States::Data(max, cur) => {
                    if self.get_current_format() == ClipboardFormat::Text {
                        if (cur as usize) < MAX_SIZE {
                            self.data.last_mut().unwrap().1[cur as usize] = *b;
                        }
                    }
                    if cur + 1 == max {
                        if self.current_index + 1 < self.num_format {
                            // There are some more formats
                            self.current_index += 1;
                            States::Format0
                        } else {
                            // No more format
                            States::End
                        }
                    } else {
                        States::Data(max, cur + 1)
                    }
                }
                States::End => unreachable!("Shouldn't reach here"),
            };
        }
    }

    fn get_current_format(&self) -> ClipboardFormat {
        self.data.last().unwrap().0
    }

    pub fn into_data(self) -> Option<Vec<u8>> {
        if self.is_empty() {
            return None;
        }
        for f in self.data.into_iter() {
            if f.0 == ClipboardFormat::Text {
                return Some(f.1);
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.num_format == 0
    }

    pub fn ended(&self) -> bool {
        matches!(self.state, States::End)
    }
}
