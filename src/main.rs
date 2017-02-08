extern crate lazy_socket;

use lazy_socket::raw::Socket as RawSocket;
use lazy_socket::raw::{
    Protocol,
    Family,
    Type,
    select
};

#[macro_use]
extern crate lazy_static;

use std::time;
use std::thread;
use std::net;
use std::process::exit;
use std::os::raw::*;
use std::sync::RwLock;

mod cli;
mod stats;
mod crtl_c;

lazy_static! {
    pub static ref STATS: RwLock<stats::Stats> = RwLock::new(stats::Stats::new());
}

///Performs TCP connection and returns tuple (is_success, duration)
fn tcp_ping(family: c_int, dest: &net::SocketAddr, timeout: u64) -> Result<(bool, time::Duration), String> {
    let ty = Type::STREAM;
    let proto = Protocol::TCP;
    let socket = match RawSocket::new(family, ty, proto) {
        Ok(socket) => socket,
        Err(error) => return Err(format!("{}", error))
    };
    let _ = socket.set_nonblocking(true);

    let now = time::Instant::now();
    let _ = socket.connect(dest);
    match select(&[], &[&socket], &[&socket], Some(timeout)) {
        Ok(num) => Ok((num != 0, now.elapsed())),
        Err(error) => Err(format!("{}", error))
    }
}

fn run() -> Result<i32, String> {
    let args = match cli::Parser::new() {
        Ok(args) => args,
        Err(error) => return Err(format!("{}", error))
    };

    if args.flags.help {
        println!("{}", args.usage());
        return Ok(0);
    }

    let interval = time::Duration::from_millis(args.options.interval);

    crtl_c::set_handler();
    let mut idx = 0 as usize;
    println!("Pinging {}/{}", args.destination.ip(), args.destination.port());
    loop {
        if !args.flags.forever && idx == args.options.number {
            break;
        }

        let (ok, elapsed) = match tcp_ping(args.options.ip_family, &args.destination, args.options.timeout) {
            Ok(result) => result,
            Err(error) => return Err(error)
        };

        let reply = match ok {
            false => format!("No reply"),
            true => format!("Reply from {}", args.destination.ip())
        };

        let elapsed_ms = (elapsed.as_secs() * 1000) as f64 + elapsed.subsec_nanos() as f64 / 1000000.0;
        println!("    {}: {} - rto={:.3}ms",
                 idx,
                 reply,
                 elapsed_ms);

        STATS.write().unwrap().add_ping(ok, elapsed_ms);
        idx += 1;
        thread::sleep(interval);
    }

    let stats = STATS.read().unwrap();
    println!("{}", *stats);
    Ok(!stats.is_ok() as i32)
}

fn main() {
    let code: i32 = match run() {
        Ok(res) => res,
        Err(error) => {
            println!("{}", error);
            1
        }
    };

    exit(code);
}

