use ::lazy_socket::raw::Socket as RawSocket;
use ::lazy_socket::raw::{
    Protocol,
    Type,
    select
};

use std::os::raw::*;
use std::net;
use std::time;

pub type FnType = fn(i32, &net::SocketAddr, u64) -> Result<(bool, time::Duration), String>;

///Performs TCP connection and returns tuple (is_success, duration)
pub fn tcp(family: c_int, dest: &net::SocketAddr, timeout: u64) -> Result<(bool, time::Duration), String> {
    let ty = Type::STREAM;
    let proto = Protocol::TCP;
    let socket = match RawSocket::new(family, ty, proto) {
        Ok(socket) => socket,
        Err(error) => return Err(format!("{}", error))
    };
    let _ = socket.set_blocking(false);

    let now = time::Instant::now();
    let _ = socket.connect(dest);
    match select(&[], &[&socket], &[&socket], Some(timeout)) {
        Ok(num) => Ok((num != 0, now.elapsed())),
        Err(error) => Err(format!("{}", error))
    }
}
