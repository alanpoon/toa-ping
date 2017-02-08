extern crate lazy_socket;

#[macro_use]
extern crate lazy_static;

use std::time;
use std::thread;
use std::process::exit;
use std::sync::RwLock;

mod cli;
mod stats;
mod crtl_c;
mod ping;

lazy_static! {
    pub static ref STATS: RwLock<stats::Stats> = RwLock::new(stats::Stats::new());
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

        let (ok, elapsed) = match (args.options.ping_fn)(args.options.ip_family, &args.destination, args.options.timeout) {
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

