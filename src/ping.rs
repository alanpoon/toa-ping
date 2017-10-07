use socket2::{Socket, Domain, Type, SockAddr, Protocol};
use std::time;
use std::net::SocketAddr;
pub type FnType = fn(&SockAddr, u64) -> Result<(bool, time::Duration), String>;

///Performs TCP connection and returns tuple (is_success, duration)
pub fn tcp(dest: &SockAddr, timeout: u64) -> Result<(bool, time::Duration), String> {
    let ty = Type::stream();
    let proto = Protocol::from(6); //tcp
    let domain = match dest.as_inet() {
        Some(_) => {
            Domain::ipv4()
        }
        None => Domain::ipv6(),
    };
    let socket = match Socket::new(domain, ty, None) {
        Ok(socket) => socket,
        Err(error) => return Err(format!("socket:new {}", error)),
    };
    let now = time::Instant::now();
    socket.connect(dest).unwrap();
    socket.set_read_timeout(Some(time::Duration::from_millis(timeout))).unwrap();
    Ok((true, now.elapsed()))
}
