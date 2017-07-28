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
    slices: Vec<&'a [u8]>,
}

impl<'a> Headers<'a> {
    fn new<'b>(bytes: &'b [u8]) -> Headers<'b> {
        let mut slices = vec![];
        let (mut start, mut nl, mut end) = (0, true, 0);
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
                }
                if !is_ws(*b) {
                    if start < i {
                        slices.push(&bytes[start..i]);
                    }
                    start = i;
                }
                nl = false;
            }
        }
        Headers { slices: slices }
    }
    pub fn iter(&self) -> std::slice::Iter<&[u8]> {
        self.slices.iter()
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
