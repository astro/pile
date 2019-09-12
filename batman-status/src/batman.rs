use std::io::Error;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub const LEDS: usize = 39;

pub struct BatmanSource {
    sock: UdpSocket,
    dest: SocketAddr,
}

impl BatmanSource {
    pub fn new<A: ToSocketAddrs>(dest: A) -> Result<BatmanSource, String> {
        let mut dests = dest.to_socket_addrs()
            .map_err(|e| format!("{}", e))?;
        match dests.next() {
            Some(dest) =>
                Ok(BatmanSource {
                    sock: UdpSocket::bind("0.0.0.0:0").unwrap(),
                    dest: dest,
                }),
            None => Err("No address".to_owned())
        }
    }
    
    pub fn send(&self, pixels: &[[u8; 3]]) -> Result<(), Error> {
        assert_eq!(pixels.len(), LEDS);
        let pkt: Vec<u8> = pixels.iter()
            .flat_map(|rgb| {
                assert_eq!(rgb.len(), 3);
                rgb
            })
            .cloned()
            .collect();
        // Send
        println!("Send {} bytes to {}", pkt.len(), self.dest);
        self.sock.send_to(&pkt[..], self.dest)?;
        Ok(())
    }
}
