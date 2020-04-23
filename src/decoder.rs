use base64;
use encoding_rs;

use self::encoding_rs::Encoding;

use std::borrow::Cow;
use std::str;

#[derive(Debug)]
enum DecoderState<'a> {
    Septet(usize),
    Cr,
    Lf,
    Wsf,
    MaybeDecode(usize),
    StartDecode,
    Charset(usize),
    ToEncoding(&'a [u8]),
    QEncoding(&'a [u8]),
    BEncoding(&'a [u8]),
    BText(&'a [u8], usize),
    QText(&'a [u8], Vec<u8>, usize),
    QStart(&'a [u8], Vec<u8>),
    QMid(&'a [u8], Vec<u8>, u8),
    QEnding(&'a [u8], Vec<u8>, usize),
    EndDecode,
}

fn hex_to_val(b: u8) -> u8 {
    if b > 96 {
        b - 87
    } else if b > 64 {
        b - 55
    } else {
        b - 48
    }
}

pub fn decode(bytes: &[u8]) -> Cow<'_, str> {
    use self::DecoderState::*;
    let orig_str = str::from_utf8(bytes).unwrap();
    let (mut new, mut state) = (Vec::new(), Septet(0));
    for (i, b) in bytes.iter().enumerate() {
        state = match (state, *b) {
            // Simple cases that don't require cloning
            (Septet(start), b'\r') => {
                new.extend(&bytes[start..i]);
                Cr
            }
            (Septet(start), b'=') => MaybeDecode(start),
            (MaybeDecode(start), b'?') => {
                new.extend(&bytes[start..i - 1]);
                StartDecode
            }
            (MaybeDecode(start), _) => Septet(start),
            // Handle whitespace folding
            (Cr, b'\n') => Lf,
            (Lf, b' ') | (Lf, b'\t') | (Wsf, b' ') | (Wsf, b'\t') => Wsf,
            (Wsf, b'=') => {
                new.push(b' ');
                MaybeDecode(i)
            }
            (Wsf, _) => {
                new.push(b' ');
                Septet(i)
            }
            (prev @ Septet(_), _) => prev,
            // RFC 2047: trigger decoding
            (StartDecode, _) => Charset(i),
            (Charset(start), b'?') => ToEncoding(&bytes[start..i]),
            (prev @ Charset(_), _) => prev,
            (ToEncoding(cset), b'Q') | (ToEncoding(cset), b'q') => QEncoding(cset),
            (ToEncoding(cset), b'B') | (ToEncoding(cset), b'b') => BEncoding(cset),
            // RFC 2047: Q encoding
            (QEncoding(cset), b'?') => QText(cset, Vec::new(), i + 1),
            (QText(cset, mut buf, start), b'_') => {
                buf.extend(&bytes[start..i]);
                buf.push(b' ');
                QText(cset, buf, i + 1)
            }
            (QText(cset, buf, start), b'?') => QEnding(cset, buf, start),
            (QEnding(cset, mut buf, start), b'=') => {
                buf.extend(&bytes[start..i - 1]);
                match Encoding::for_label(cset) {
                    Some(enc) => {
                        new.extend(enc.decode(&buf).0.as_ref().bytes());
                        Septet(i + 1)
                    }
                    None => panic!(
                        "unknown encoding {:?} from {:?}",
                        str::from_utf8(cset).unwrap(),
                        orig_str
                    ),
                }
            }
            (QEnding(cset, buf, start), b'?') => QEnding(cset, buf, start),
            (QEnding(cset, buf, start), _) => QText(cset, buf, start),
            (QText(cset, mut buf, start), b'=') => {
                buf.extend(&bytes[start..i]);
                QStart(cset, buf)
            }
            (QStart(cset, buf), b) => QMid(cset, buf, b),
            (QMid(cset, mut buf, x), y) => {
                buf.push((hex_to_val(x) << 4) + hex_to_val(y));
                QText(cset, buf, i + 1)
            }
            (prev @ QText(_, _, _), _) => prev,
            // RFC 2047: B encoding
            (BEncoding(cset), b'?') => BText(cset, i + 1),
            (BText(cset, start), b'?') => {
                let buf = &bytes[start..i];
                let binary = base64::decode_config(buf, base64::IMAP_MUTF7).unwrap();
                match Encoding::for_label(cset) {
                    Some(enc) => {
                        new.extend(enc.decode(&binary).0.as_ref().bytes());
                        EndDecode
                    }
                    None => panic!("unknown encoding {}", str::from_utf8(cset).unwrap()),
                }
            }
            (prev @ BText(_, _), _) => prev,
            (EndDecode, b'=') => Septet(i + 1),
            // Panic for all transitions not described yet
            prev => panic!(
                "incorrect state transition (transforming): {:?} {:?}",
                prev, orig_str
            ),
        };
    }
    match state {
        Septet(0) | MaybeDecode(0) => Cow::Borrowed(orig_str),
        Septet(start) | MaybeDecode(start) => {
            new.extend(&bytes[start..]);
            Cow::Owned(String::from_utf8(new).unwrap())
        }
        prev => panic!("unexpected end state {:?} {:?}", prev, orig_str),
    }
}

#[cfg(test)]
mod tests {
    use super::decode;
    use super::Cow;

    macro_rules! expect_variant {
        ( $input:expr, $variant:ident, $expect:expr ) => {
            match decode($input) {
                Cow::$variant(bytes) => assert_eq!(bytes, $expect),
                d => panic!("incorrect variant {:?}", d),
            }
        };
    }

    #[test]
    fn simple() {
        expect_variant!(b"abc", Borrowed, "abc");
        expect_variant!(b"=foo", Borrowed, "=foo");
        expect_variant!(b"====", Borrowed, "====");
    }

    #[test]
    fn folding() {
        expect_variant!(b"ab\r\n    c", Owned, "ab c");
    }

    #[test]
    fn rfc2047_quoted() {
        expect_variant!(
            b"=?UTF-8?Q?Foo_=C3=87ar?= <baz@example.org>",
            Owned,
            "Foo Çar <baz@example.org>"
        );
        expect_variant!(
            b"=?UTF-8?Q?Paper_R=c3=bcck?= crud",
            Owned,
            "Paper Rück crud"
        );
        expect_variant!(b"=?ISO-8859-1?Q?Question??=", Owned, "Question?");
    }

    #[test]
    fn rfc2047_b64() {
        expect_variant!(
            b"=?UTF-8?B?ScOxdMOrcm7DonRpw7Ruw6BsaXrDpnRpw7hu?=",
            Owned,
            "Iñtërnâtiônàlizætiøn"
        );
        expect_variant!(
            b"=?utf-8?B?SW50ZXJu?=\r\n =?utf-8?Q?foo?=",
            Owned,
            "Intern foo"
        );
    }
}
