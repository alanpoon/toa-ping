use std::env;
use std::fmt;
use std::net;
use std::net::ToSocketAddrs;
use std::str::FromStr;

const USAGE: &'static str = "usage: toa-ping [flags] [options] <destination>

Performs ping toward destination.

Flags:
  -h, --help    - Prints this message.
  -f, --forever - Keep going forever.

Options:
  -n <number>   - Number of pings to send. Default is 4.
  -i <interval> - Time interval between pings in milliseconds. Default is 500.
  -w <timeout>  - Timeout to wait for each response in milliseconds. Default 1000ms.
";

pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == "" {
            write!(f, "{}", USAGE)
        }
        else {
            write!(f, "{}\n{}", self.0, USAGE)
        }
    }
}

#[derive(Default)]
pub struct Flags {
    pub forever: bool,
    pub help: bool
}

#[derive(Default)]
pub struct Options {
    pub number: usize,
    pub interval: u64,
    pub timeout: u64
}

pub struct Parser {
    pub flags: Flags,
    pub options: Options,
    pub destination: net::SocketAddr
}

fn parse_next_int<T: FromStr>(arg: Option<String>, opt_name: &str) -> Result<T, ParseError> {
    if let Some(num) = arg {
        if let Ok(num) = num.parse::<T>() {
            Ok(num)
        }
        else {
            Err(ParseError(format!("Invalid number {} is supplied for option {}", num, opt_name)))
        }
    }
    else {
        Err(ParseError(format!("Missing value for option {}", opt_name)))
    }
}

impl Parser {
    pub fn new() -> Result<Parser, ParseError> {
        let mut flags = Flags::default();
        let mut options = Options::default();
        let mut destination: Option<net::SocketAddr> = None;

        let mut args = env::args().skip(1);

        options.number = 4;
        options.interval = 500;
        options.timeout = 1000;

        while let Some(arg) = args.next() {
            let arg = arg.as_ref();
            match arg {
                "-h" | "--help" => flags.help = true,
                "-f" | "--forever" => flags.forever = true,
                opt @ "-n" => {
                    match parse_next_int(args.next(), opt) {
                        Ok(num) => options.number = num,
                        Err(error) => return Err(error)
                    }
                },
                opt @ "-i" => {
                    match parse_next_int(args.next(), opt) {
                        Ok(num) => options.interval = num,
                        Err(error) => return Err(error)
                    }
                },
                opt @ "-w" => {
                    match parse_next_int(args.next(), opt) {
                        Ok(num) => options.timeout = num,
                        Err(error) => return Err(error)
                    }
                }
                dest @ _ => {
                    let mut addrs = match dest.to_socket_addrs() {
                        Ok(iter) => iter,
                        Err(_) => return Err(ParseError(format!("Invalid destination {}", dest)))
                    };
                    destination = addrs.next();
                }
            }
        }

        if destination.is_none() {
            if flags.help {
                return Err(ParseError("".to_string()));
            }
            else {
                return Err(ParseError("Destination is not supplied".to_string()));
            }
        }

        Ok(Parser {
            flags: flags,
            options: options,
            destination: destination.unwrap()
        })
    }

    pub fn usage(&self) -> &'static str {
        return USAGE;
    }
}
