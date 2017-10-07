extern crate socket2;
#[macro_use]
extern crate lazy_static;

pub mod cli;
pub mod stats;
pub mod crtl_c;
pub mod ping;
use std::net;
use std::thread;
use std::time;
use std::net::ToSocketAddrs;
use socket2::SockAddr;
use std::sync::RwLock;
lazy_static! {
    pub static ref STATS: RwLock<stats::Stats> = RwLock::new(stats::Stats::new());
}
pub enum Family {
    IPv4,
    IPv6,
    Other,
    None,
}
pub fn run(dest: &'static str) -> Result<i32, String> {
    let mut destination: Option<net::SocketAddr> = None;
    let mut destination4: Option<net::SocketAddr> = None;
    let mut destination6: Option<net::SocketAddr> = None;
    let addrs = match dest.to_socket_addrs() {
        Ok(iter) => iter,
        Err(_) => {
            if let Ok(addrs) = (dest, 0).to_socket_addrs() {
                addrs
            } else {
                return Err(format!("Invalid destination {}", dest));
            }
        }
    };
    for dest in addrs {
        if destination.is_none() {
            destination = Some(dest);
        }
        match dest {
            net::SocketAddr::V4(_) => {
                if destination4.is_none() {
                    destination4 = Some(dest)
                }
            }
            net::SocketAddr::V6(_) => {
                if destination6.is_none() {
                    destination6 = Some(dest)
                }
            }
        }
        if destination.is_none() {
            return Err("Failed to resolve anything from destination :(".to_string());
        }
    }
    let timeout = 3000;
    let mut ip_family = Family::IPv4;
    let ping_fn = ping::tcp;
    let interval = time::Duration::from_millis(500);
    let mut idx = 0 as usize;
    let mut destination = match ip_family {
        Family::IPv4 => {
            match destination4 {
                Some(dest) => dest,
                None => {
                    return Err("IPv4 address is not found. Cannot ping with this version >.<"
                                   .to_string())
                }
            }
        }
        Family::IPv6 => {
            match destination6 {
                Some(dest) => dest,
                None => {
                    return Err("IPv6 address is not found. Cannot ping with this version >.<"
                                   .to_string())
                }
            }
        }
        _ => {
            let destination = destination.unwrap();
            ip_family = match destination {
                net::SocketAddr::V4(_) => Family::IPv4,
                net::SocketAddr::V6(_) => Family::IPv6,
            };

            destination
        }
    };
    if destination.port() == 0 {
        destination.set_port(80);
    }

    loop {

        if idx == 4 {
            break;
        }
        let destination = SockAddr::from(destination);
        let (ok, elapsed) = match (ping_fn)(&destination, timeout) {
            Ok(result) => result,
            Err(error) => return Err(error),
        };
        let reply = match ok {
            false => format!("No reply"),
            true => format!("Reply from {:?}", destination.as_ptr()),
        };

        let elapsed_ms = (elapsed.as_secs() * 1000) as f64 +
                         elapsed.subsec_nanos() as f64 / 1000000.0;
        println!("    {}: {} - rto={:.3}ms", idx, reply, elapsed_ms);
        STATS.write().unwrap().add_ping(ok, elapsed_ms);
        idx += 1;
        thread::sleep(interval);

    }
    let stats = STATS.read().unwrap();
    Ok(!stats.is_ok() as i32)
}
