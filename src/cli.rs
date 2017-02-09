use ::lazy_socket::raw::{
    Family
};

use std::env;
use std::fmt;
use std::net;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::os::raw::*;

const USAGE: &'static str = "usage: toa-ping [flags] [options] <destination>

Performs ping toward destination.

Destination format: <host>[:<port>]

Flags:
  -h, --help    - Prints this message.
  -f, --forever - Keep going forever.

Options:
  -p <protocol> - Specifies protocol to use. Default is tcp.
  -n <number>   - Number of pings to send. Default is 4.
  -i <interval> - Time interval between pings in milliseconds. Default is 500.
  -w <timeout>  - Time to wait for each response in milliseconds. Default is 1000.
  -4            - Enforce IPv4 version. Default is first resolved address.
  -6            - Enforce IPv6 version. Default is first resolved address.

Supported protocols:
  tcp - Measures RTT of connection establishment.
";

pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == "" {
            write!(f, "{}", USAGE)
        }
        else {
            write!(f, "ERROR:{}\n\n{}", self.0, USAGE)
        }
    }
}

#[derive(Default)]
pub struct Flags {
    pub forever: bool,
    pub help: bool
}

pub struct Options {
    pub number: usize,
    pub interval: u64,
    pub timeout: u64,
    pub ip_family: c_int,
    pub ping_fn: ::ping::FnType
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
        let mut options = Options {
            number: 4,
            interval: 500,
            timeout: 1000,
            ip_family: 0,
            ping_fn: ::ping::tcp
        };
        let mut destination: Option<net::SocketAddr> = None;
        let mut destination4: Option<net::SocketAddr> = None;
        let mut destination6: Option<net::SocketAddr> = None;
        let mut protocol_fn: Option<::ping::FnType> = None;

        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            let arg = arg.as_ref();
            match arg {
                "-h" | "--help" => flags.help = true,
                "-f" | "--forever" => flags.forever = true,
                "-4" => options.ip_family = Family::IPV4,
                "-6" => options.ip_family = Family::IPV6,
                "-p" => {
                    let value = args.next();
                    match value.as_ref().map(String::as_str) {
                        Some("tcp") => protocol_fn = Some(::ping::tcp),
                        arg @ Some(_) => return Err(ParseError(format!("Invalid protocol {}", arg.unwrap()))),
                        _ => return Err(ParseError("Protocol hasn't been supplied".to_string()))
                    }
                },
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
                    let addrs = match dest.to_socket_addrs() {
                        Ok(iter) => iter,
                        Err(_) => {
                            if let Ok(addrs) = (dest, 0).to_socket_addrs() {
                                addrs
                            }
                            else {
                                return Err(ParseError(format!("Invalid destination {}", dest)))
                            }
                        }
                    };

                    for dest in addrs {
                        if destination.is_none() {
                            destination = Some(dest);
                        }

                        match dest {
                            net::SocketAddr::V4(_) => if destination4.is_none() { destination4 = Some(dest) },
                            net::SocketAddr::V6(_) => if destination6.is_none() { destination6 = Some(dest) }
                        }
                    }

                    if destination.is_none() {
                        return Err(ParseError("Failed to resolve anything from destination :(".to_string()));
                    }
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

        let mut destination = match options.ip_family {
            Family::IPV4 => {
                match destination4 {
                    Some(dest) => dest,
                    None => return Err(ParseError("IPv4 address is not found. Cannot ping with this version >.<".to_string()))
                }
            },
            Family::IPV6 => {
                match destination6 {
                    Some(dest) => dest,
                    None => return Err(ParseError("IPv6 address is not found. Cannot ping with this version >.<".to_string()))
                }
            },
            _ => {
                let destination = destination.unwrap();
                options.ip_family = match destination {
                    net::SocketAddr::V4(_) => Family::IPV4,
                    net::SocketAddr::V6(_) => Family::IPV6
                };
                destination
            }
        };

        if destination.port() == 0 {
            destination.set_port(80);
        }

        options.ping_fn = protocol_fn.unwrap_or(::ping::tcp);

        Ok(Parser {
            flags: flags,
            options: options,
            destination: destination
        })
    }

    pub fn usage(&self) -> &'static str {
        return USAGE;
    }
}
