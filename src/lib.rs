use indexmap;

use indexmap::IndexMap;

use std::borrow::Cow;
use std::str;

mod decoder;

pub struct Message<'a> {
    bytes: &'a [u8],
}

impl<'a> Message<'a> {
    pub fn from_slice(bytes: &[u8]) -> Message<'_> {
        Message { bytes }
    }
    pub fn headers<'s>(&'s self) -> Headers<'s> {
        Headers::new(self.bytes)
    }
}

pub struct Headers<'a> {
    map: IndexMap<String, Vec<&'a [u8]>>,
}

#[derive(Debug)]
enum HeaderState<'a> {
    Key(usize),
    Colon(&'a str),
    Value(&'a str, usize),
    Ending(&'a str, usize),
    Lf(&'a str, usize),
}

impl<'a> Headers<'a> {
    fn new(bytes: &[u8]) -> Headers<'_> {
        let mut map = IndexMap::new();
        use HeaderState::*;
        let mut state = Key(0);
        for (i, b) in bytes.iter().enumerate() {
            state = match (state, *b) {
                (Key(start), b':') => Colon(str::from_utf8(&bytes[start..i]).unwrap()),
                (prev @ Key(_), _) => prev,
                (prev @ Colon(_), b' ') | (prev @ Colon(_), b'\t') => prev,
                (Colon(key), _) => Value(key, i),
                (Value(key, start), b'\n') => Lf(key, start),
                (prev @ Value(_, _), _) => prev,
                (Lf(key, start), b'\r') => Ending(key, start),
                (Lf(key, start), b' ') | (Lf(key, start), b'\t') => Value(key, start),
                (Lf(key, start), _) => {
                    let values = map.entry(key.to_lowercase()).or_insert(vec![]);
                    values.push(&bytes[start..i - 2]);
                    Key(i)
                }
                (Ending(key, start), b'\n') => {
                    let values = map.entry(key.to_lowercase()).or_insert(vec![]);
                    values.push(&bytes[start..i - 3]);
                    break;
                }
                prev => panic!("invalid state transition {:?}", prev),
            };
        }
        Headers { map }
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, String, Vec<&[u8]>> {
        self.map.iter()
    }
    pub fn get(&self, key: &str) -> Vec<Cow<'_, str>> {
        let values = match self.map.get(&key.to_lowercase()) {
            None => {
                return Vec::new();
            }
            Some(vals) => vals,
        };
        values.iter().map(|s| decoder::decode(&s)).collect()
    }
    pub fn get_first(&self, key: &str) -> Option<Cow<'_, str>> {
        let mut res = None;
        let mut vec = self.get(key);
        for val in vec.drain(..) {
            res = Some(val);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::Headers;

    #[test]
    fn simple() {
        let h = Headers::new(b"X: foo\r\nY: bar\r\n\r\nbody");
        assert_eq!(h.map.get("x"), Some(&vec![b"foo".as_ref()]));
        assert_eq!(h.map.get("y"), Some(&vec![b"bar".as_ref()]));
    }

    #[test]
    fn no_body() {
        let h = Headers::new(b"X: foo\r\nY: bar\r\n\r\n");
        assert_eq!(h.map.get("x"), Some(&vec![b"foo".as_ref()]));
        assert_eq!(h.map.get("y"), Some(&vec![b"bar".as_ref()]));
    }

    #[test]
    fn folding() {
        let h = Headers::new(b"X: foo\r\n \tbar\r\n\r\n");
        assert_eq!(h.map.get("x"), Some(&vec![b"foo\r\n \tbar".as_ref()]));
    }
}
