extern crate ordermap;

use ordermap::OrderMap;

use std::str;

pub struct Message<'a> {
    bytes: &'a [u8],
}

impl<'a> Message<'a> {
    pub fn from_slice<'b>(bytes: &'b [u8]) -> Message<'b> {
        Message { bytes }
    }
    pub fn headers<'s>(&'s self) -> Headers<'s> {
        Headers::new(self.bytes)
    }
}

pub struct Headers<'a> {
    map: OrderMap<String, Vec<&'a [u8]>>,
}

impl<'a> Headers<'a> {
    fn new<'b>(bytes: &'b [u8]) -> Headers<'b> {
        let mut map = OrderMap::new();
        let (mut nl, mut end, mut key_start, mut key_end, mut val_start) = (true, 0, 0, 0, 0);
        for (i, b) in bytes.iter().enumerate() {
            if *b == b'\n' {
                nl = true;
                if end == 0 {
                    end = 1;
                } else if end == 2 {
                    break;
                }
            } else if nl {
                if end == 1 && *b == b'\r' {
                    end = 2;
                } else if !is_ws(*b) {
                    if key_start < i {
                        if key_end > 0 {
                            let key = str::from_utf8(&bytes[key_start..key_end]).unwrap();
                            let values = map.entry(key.to_lowercase()).or_insert(vec![]);
                            values.push(&bytes[val_start..i - 2]);
                        } else {
                            panic!("found header without discernible key");
                        }
                    }
                    key_start = i;
                    key_end = 0;
                }
                nl = false;
            } else if key_end == 0 && *b == b':' {
                key_end = i;
                val_start = i + 1;
            } else if i == val_start && is_ws(*b) {
                val_start = i + 1;
            }
        }
        Headers { map: map }
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn iter(&self) -> ordermap::Iter<String, Vec<&[u8]>> {
        self.map.iter()
    }
}

fn is_ws(b: u8) -> bool {
    unsafe { ASCII.get_unchecked(b as usize) & SPACE != 0 }
}

static ASCII: [u8; 256] = [
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, SPACE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	SPACE, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT,
	PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, PRINT, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
	NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
];

const NONE: u8 = 0b00;
const PRINT: u8 = 0b01;
const SPACE: u8 = 0b10;
