extern crate toa_ping;

fn main() {
    let u = toa_ping::run("www.google.com");
    if let Ok(i) = u {
        println!("kkk{}", i);
    }
}
