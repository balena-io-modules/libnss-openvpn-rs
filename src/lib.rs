extern crate libc;
extern crate glob;

pub mod glue;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::Ipv4Addr;

static GLOB_PATTERN: &str = "/var/run/openvpn/server-*.status";
static SUFFIX: &str = ".balena";

#[allow(dead_code)]
#[derive(Debug)]
pub enum NssStatus {
    TryAgain = -2,
    Unavailable,
    NotFound,
    Success,
}

fn lookup_client_ip<T: BufRead>(hostname: &str, buf: T) -> Result<Ipv4Addr, NssStatus> {
	for line in buf.lines()
		.filter_map(|l| l.ok())
		.skip_while(|l| l != "ROUTING TABLE")
		.skip(2)
		.take_while(|l| l != "GLOBAL STATS") {
		let mut parts = line.split(',');
		let ip_part = parts.next().unwrap().parse::<Ipv4Addr>().unwrap();
		let hostname_part = parts.next().unwrap();

		if hostname_part == hostname {
			return Ok(ip_part);
		}
	}

	Err(NssStatus::NotFound)
}

fn gethostbyname(name: &str) -> Result<Ipv4Addr, NssStatus> {
    if !name.ends_with(SUFFIX) {
        return Err(NssStatus::NotFound);
    }
    let name = &name[..name.len() - SUFFIX.len()];

    for entry in glob::glob(GLOB_PATTERN).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            let reader = match File::open(path) {
                Ok(f) => BufReader::new(f),
                Err(_) => return Err(NssStatus::Unavailable),
            };

	        match lookup_client_ip(name, reader) {
		        Ok(ip) => return Ok(ip),
		        Err(_) => continue,
	        };
        }
    }

    Err(NssStatus::NotFound)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Cursor;

	fn get_cfg() -> Cursor<String> {
		let mut buf = Cursor::new(String::from("\
OpenVPN CLIENT LIST
Updated,Tue Nov  6 14:07:08 2018
Common Name,Real Address,Bytes Received,Bytes Sent,Connected Since
0123456789abcdef0002,127.0.0.1:12345,0,0,Wed Nov 14 00:46:00 2018
0123456789abcdef0003,127.0.0.1:54321,0,0,Wed Nov 14 00:46:00 2018
ROUTING TABLE
Virtual Address,Common Name,Real Address,Last Ref
10.240.1.2,0123456789abcdef0002,127.0.0.1:12345,Wed Nov 14 00:46:00 2018
10.240.1.3,0123456789abcdef0003,127.0.0.1:54321,Wed Nov 14 00:46:00 2018
GLOBAL STATS
Max bcast/mcast queue length,0
END"));
		buf.set_position(0);
		buf
	}

	#[test]
	fn test_unknown() {
		let res = lookup_client_ip("0123456789abcdef0001", get_cfg());
		assert!(res.is_err(), "unknown hostname should return error");
	}

	#[test]
	fn test_known() {
		let ip = lookup_client_ip("0123456789abcdef0002", get_cfg()).unwrap();
		assert_eq!(ip.to_string(), "10.240.1.2", "invalid ip returned");

		let ip = lookup_client_ip("0123456789abcdef0003", get_cfg()).unwrap();
		assert_eq!(ip.to_string(), "10.240.1.3", "invalid ip returned");
	}
}
