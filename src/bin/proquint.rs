extern crate proquint;

use std::env;
use std::process::exit;
use std::str::FromStr;
use proquint::{AsProquint, Proquint};
use std::net::Ipv4Addr;

fn exit_usage() -> ! {
        println!("Usage: proquint u64:<int>");
        println!("       proquint u32:<int>");
        println!("       proquint u16:<int>");
        println!("       proquint proquint:<proquint>");
        exit(1);
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        exit_usage();
    }

    let inp = args.nth(1).unwrap();
    let p = if inp.starts_with("u64:") {
        u64::from_str(&inp[4..]).unwrap().as_proquint()
    } else if inp.starts_with("u32:") {
        u32::from_str(&inp[4..]).unwrap().as_proquint()
    } else if inp.starts_with("u16:") {
        u16::from_str(&inp[4..]).unwrap().as_proquint()
    } else if inp.starts_with("ip:") {
        let ip = Ipv4Addr::from_str(&inp[3..]).unwrap();
        let o = ip.octets();
        Proquint::from_slice(&[
                             ((o[0] as u16) << 8) | o[1] as u16,
                             ((o[2] as u16) << 8) | o[3] as u16
                     ])
    } else if inp.starts_with("proquint:") {
        let p = Proquint::from_str(&inp[9..]).unwrap();
        println!("{:?}", p.to_ints());
        exit(0);
    } else {
        exit_usage();
    };

    println!("{}", p);
}
