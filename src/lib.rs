extern crate libc;
extern crate glob;

pub mod glue;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::Ipv4Addr;
use glob::glob;

static GLOB_PATTERN: &str = "/var/run/openvpn/server-*.status";
static SUFFIX: &str = ".vpn";

#[allow(dead_code)]
pub enum NssStatus {
    TryAgain = -2,
    Unavailable,
    NotFound,
    Success,
}

fn gethostbyname(name: &str) -> Result<Ipv4Addr, NssStatus> {
    if !name.ends_with(SUFFIX) {
        return Err(NssStatus::NotFound);
    }
    let name = &name[..name.len() - SUFFIX.len()];

    for entry in glob(GLOB_PATTERN).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            let reader = match File::open(path) {
                Ok(f) => BufReader::new(f),
                Err(_) => return Err(NssStatus::Unavailable),
            };

            let routing_table = reader
                .lines()
                .filter_map(|l| l.ok())
                .skip_while(|l| l != "ROUTING TABLE")
                .skip(2)
                .take_while(|l| l != "GLOBAL STATS");

            for line in routing_table {
                let mut parts = line.split(',');
                let ip = parts.next().unwrap().parse::<Ipv4Addr>().unwrap();
                let hostname = parts.next().unwrap();

                if hostname == name {
                    return Ok(ip);
                }
            }
        }
    }

    Err(NssStatus::NotFound)
}

#[cfg(test)]
mod tests {
    use gethostbyname;

    #[test]
    fn it_works() {
        match gethostbyname("94e0cc936f84005d27ed74a430713884.vpn") {
            Ok(t) => println!("Success {:?}", t),
            Err(_) => println!("Failed"),
        }
    }
}
