//! A Rust implementation of [Proquints](http://arxiv.org/html/0901.4016),
//! Readable, Spellable, and Pronounceable identifiers, Proquints represent unsigned 16bit
//! integers as a 5 letter label.
//!
//! # Usage
//!
//! Generate a Proquint from one or more u16
//!
//! ```
//! use proquint::Proquint;
//! println!("{}", Proquint::from_slice(&[127]));
//! ```
//!
//! The `AsProquint` trait provides convertion methods over existing types
//!
//! ```
//! use proquint::AsProquint;
//! println!("{}", 127u16.as_proquint());
//! ```
//!
//! Convert a String into a Proquint
//!
//! ```
//! use proquint::Proquint;;
//! use std::str::FromStr;
//! let p = Proquint::from_str("tobog-higil").unwrap();
//! ```

use std::iter::FromIterator;
use std::fmt;
use std::str;
use std::str::FromStr;
use std::net::Ipv4Addr;

const UINT2CONSONANT: &'static [u8] = b"bdfghjklmnprstvz";
const UINT2VOWEL: &'static [u8] = b"aiou";

fn quint_to_ascii(u: u16, out: &mut Vec<u8>) {
    out.reserve(5);
    let con0 = ((0xf000 & u) >> 12) as usize;
    debug_assert!(con0 < UINT2CONSONANT.len());
    out.push(UINT2CONSONANT[con0]);

    let vo1 = ((0x0c00 & u) >> 10) as usize;
    debug_assert!(vo1 < UINT2VOWEL.len());
    out.push(UINT2VOWEL[vo1]);

    let con2 = ((0x03c0 & u) >> 6) as usize;
    debug_assert!(con2 < UINT2CONSONANT.len());
    out.push(UINT2CONSONANT[con2]);

    let vo3 = ((0x0030 & u) >> 4) as usize;
    debug_assert!(vo3 < UINT2VOWEL.len());
    out.push(UINT2VOWEL[vo3]);

    let con4 = (0x000f & u) as usize;
    debug_assert!(con4 < UINT2CONSONANT.len());
    out.push(UINT2CONSONANT[con4]);
}

#[derive(Debug, PartialEq)]
pub struct Proquint {
    /// Internally store ASCII chars
    inner: Vec<u8>,
}

impl Proquint {
    pub fn from_vec(ints: Vec<u16>) -> Proquint {
        let mut v = Vec::with_capacity(ints.len()*5);
        for int in ints {
            quint_to_ascii(int, &mut v);
        }
        Proquint { inner: v }
    }

    pub fn from_slice(ints: &[u16]) -> Proquint {
        let mut v = Vec::with_capacity(ints.len()*5);
        for int in ints {
            quint_to_ascii(*int, &mut v);
        }
        Proquint { inner: v }
    }

    pub fn append(&mut self, u: u16) {
        quint_to_ascii(u, &mut self.inner);
    }

    /// Add a label to the Proquint from 5 ASCII chars
    fn append_label(&mut self, label: &[u8]) -> Result<(), ProquintError> {
        if label.len() != 5 {
            return Err(ProquintError::InvalidLabelLength)
        }

        if UINT2CONSONANT.contains(&label[0]) {
            self.inner.push(label[0]);
        } else {
            return Err(ProquintError::InvalidConsonant(label[0]));
        }
        if UINT2VOWEL.contains(&label[1]) {
            self.inner.push(label[1]);
        } else {
            return Err(ProquintError::InvalidVowel(label[1]));
        }
        if UINT2CONSONANT.contains(&label[2]) {
            self.inner.push(label[2]);
        } else {
            return Err(ProquintError::InvalidConsonant(label[2]));
        }
        if UINT2VOWEL.contains(&label[3]) {
            self.inner.push(label[3]);
        } else {
            return Err(ProquintError::InvalidVowel(label[3]));
        }
        if UINT2CONSONANT.contains(&label[4]) {
            self.inner.push(label[4]);
        } else {
            return Err(ProquintError::InvalidConsonant(label[4]));
        }
        Ok(())
    }

//    /// Convert Proquint to bytes
//    pub fn to_bytes(&self) -> Vec<u8> {
//        self.to_ints().iter()
//    }

    /// Convert a Proquint to binary
    pub fn to_ints(&self) -> Vec<u16> {
        let mut v = Vec::with_capacity(self.inner.len()/5);

        for label in self.inner.chunks(5) {
            let mut val = 0u16;
            for c in label {
                if UINT2CONSONANT.contains(c) {
                    val <<= 4;
                    val += UINT2CONSONANT.iter().position(|&x| x == *c).unwrap() as u16;
                } else if UINT2VOWEL.contains(c) {
                    val <<= 2;
                    val += UINT2VOWEL.iter().position(|&x| x == *c).unwrap() as u16;
                } else {
                    panic!("BUG: found invalid chars in Proquint");
                }
            }
            v.push(val);
        }
        v
    }
}

impl FromIterator<u16> for Proquint {
    fn from_iter<T>(it: T) -> Self 
        where T: IntoIterator<Item=u16> {
        let mut v = Vec::with_capacity(5);
        for int in it {
            quint_to_ascii(int, &mut v);
        }
        Proquint { inner: v }
    }
}

#[derive(Debug, PartialEq)]
pub enum ProquintError {
    InvalidLabelLength,
    InvalidConsonant(u8),
    InvalidVowel(u8),
}

impl FromStr for Proquint {
    type Err = ProquintError;

    fn from_str(s: &str) -> Result<Proquint, Self::Err> {
        let v = Vec::with_capacity(s.len());
        let mut p = Proquint { inner: v };

        if s.contains('-') {
            for label in s.split('-') {
                if label.len() != 5 {
                    return Err(ProquintError::InvalidLabelLength)
                }
                try!(p.append_label(label.as_bytes()));
            }
        } else {
            for label in s.as_bytes().chunks(5) {
                try!(p.append_label(label));
            }
        }
        Ok(p)
    }
}

impl fmt::Display for Proquint {
    fn fmt(&self, fm: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for chunk in self.inner.chunks(5) {
            let s = unsafe { str::from_utf8_unchecked(chunk) };
            if first {
                first = false;
                try!(write!(fm, "{}", s));
            } else {
                try!(write!(fm, "-{}", s));
            }
        }
        Ok(())
    }
}

pub trait AsProquint {
    /// Appends this to an existing Proquint
    fn into_proquint(&self, &mut Proquint);
    /// Convert into a Proquint
    fn as_proquint(&self) -> Proquint {
        let mut p = Proquint::from_vec(vec![]);
        self.into_proquint(&mut p);
        p
    }
    /// Convenience method, to generate Proquint string
    fn as_proquint_str(&self) -> String {
        self.as_proquint().to_string()
    }
}

impl AsProquint for u16 {
    fn into_proquint(&self, to: &mut Proquint) {
        to.append(*self);
    }
}
impl AsProquint for u32 {
    fn into_proquint(&self, to: &mut Proquint) {
        to.append(((self & 0xffff0000) >> 16) as u16);
        to.append((self & 0x0000ffff) as u16);
    }
}
impl<T: AsProquint> AsProquint for Vec<T> {
    fn into_proquint(&self, mut to: &mut Proquint) {
        for item in self {
            item.into_proquint(to)
        }
    }
}
impl<T: AsProquint> AsProquint for [T] {
    fn into_proquint(&self, mut to: &mut Proquint) {
        for item in self {
            item.into_proquint(to);
        }
    }
}
impl AsProquint for u64 {
    fn into_proquint(&self, to: &mut Proquint) {
        to.append(((self & 0xffff000000000000) >> 48) as u16);
        to.append(((self & 0x0000ffff00000000) >> 32) as u16);
        to.append(((self & 0x00000000ffff0000) >> 16) as u16);
        to.append((self & 0x000000000000ffff) as u16);
    }
}
impl AsProquint for Ipv4Addr {
    fn into_proquint(&self, to: &mut Proquint) {
        let o = self.octets();
        to.append(((o[0] as u16) << 8) | o[1] as u16);
        to.append(((o[2] as u16) << 8) | o[3] as u16);
    }
}

#[test]
fn sanity() {
    assert_eq!(UINT2CONSONANT.len(), 16);
    assert_eq!(UINT2VOWEL.len(), 4);
}

#[test]
fn test_proquint_u16() {
    let p = 1u16.as_proquint();
    let s = p.to_string();
    assert_eq!(s, "babad");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [ 1]);

    let p = 0u16.as_proquint();
    let s = p.to_string();
    assert_eq!(s, "babab");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [0]);

    let p = 0xffffu16.as_proquint();
    let s = p.to_string();
    assert_eq!(s, "zuzuz");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [0xffff]);
}

#[test]
fn test_proquint_u32() {
    let p = 1u32.as_proquint();
    let s = p.to_string();
    assert_eq!(s, "babab-babad");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [0, 1]);

    let p = 0u32.as_proquint();
    let s = p.to_string();
    assert_eq!(s, "babab-babab");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [0, 0]);
}

#[test]
fn test_proquint_u64() {
    let p = 1u64.as_proquint();
    let s = p.to_string();
    assert_eq!(s, "babab-babab-babab-babad");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [0, 0, 0, 1]);

    let p = 0u64.as_proquint();
    let s = p.to_string();
    assert_eq!(s, "babab-babab-babab-babab");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [0, 0, 0, 0]);
}

#[test]
fn test_proquint_vec() {
    let p = vec![0u16, 1].as_proquint();
    let s = p.to_string();
    assert_eq!(s, "babab-babad");
    assert_eq!(p, Proquint::from_str(&s).unwrap());
    assert_eq!(p.to_ints(), [0u16, 1]);
}

#[test]
fn test_proquint_from_ip() {
    let p = Ipv4Addr::from_str("127.0.0.1").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "lusab-babad");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("63.84.220.193").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "gutih-tugad");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("63.118.7.35").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "gutuk-bisog");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("140.98.193.141").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "mudof-sakat");

    let p = Ipv4Addr::from_str("64.255.6.200").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "haguz-biram");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("128.30.52.45").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "mabiv-gibot");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("147.67.119.2").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "natag-lisaf");

    let p = Ipv4Addr::from_str("212.58.253.68").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "tibup-zujah");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("216.35.68.215").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "tobog-higil");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("216.68.232.21").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "todah-vobij");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("198.81.129.136").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "sinid-makam");
    assert_eq!(p, Proquint::from_str(&s).unwrap());

    let p = Ipv4Addr::from_str("12.110.110.204").unwrap().as_proquint();
    let s = p.to_string();
    assert_eq!(s, "budov-kuras");
}

#[test]
fn test_from_string() {
    assert_eq!(Proquint::from_str("XXX"), Err(ProquintError::InvalidLabelLength));
    assert_eq!(Proquint::from_str("XXXXX"), Err(ProquintError::InvalidConsonant(b'X')));
    assert_eq!(Proquint::from_str("bbbbb"), Err(ProquintError::InvalidVowel(b'b')));
    Proquint::from_str("babab").unwrap();

    let p0 = Proquint::from_str("babab-babab").unwrap();
    let p1 = Proquint::from_str("bababbabab").unwrap();
    assert_eq!(p0, p1);
}
