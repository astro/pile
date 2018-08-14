use managed::ManagedSlice;
use smoltcp::Error;
use smoltcp::time::{Instant, Duration};
use smoltcp::socket::{SocketHandle, SocketSet,
                      UdpSocket, UdpSocketBuffer};
use smoltcp::wire::IpEndpoint;
use smoltcp::storage::{PacketMetadata, PacketBuffer};


const PRIO_TIMEOUT: u64 = 500;
const CMD_SET_PIXEL_COLORS: u8 = 0;


pub struct Receiver {
    handle: SocketHandle,
    current_prio: u8,
    last_packet: Instant,
}

impl Receiver {
    pub fn new<'a: 'b, 'b: 'a, MS, PS, E>(endpoint: E, sockets: &mut SocketSet<'_, 'a, 'b>, rx_buffer: (MS, PS), tx_buffer: (MS, PS)) -> Self
    where E: Into<IpEndpoint>,
          MS: Into<ManagedSlice<'a, PacketMetadata<IpEndpoint>>>,
          PS: Into<ManagedSlice<'b, u8>>,
    {
        let rx_buffer: UdpSocketBuffer<'a, 'a> = PacketBuffer::new(
            rx_buffer.0.into(), rx_buffer.1.into()
        );
        let tx_buffer: UdpSocketBuffer<'a, 'a> = PacketBuffer::new(
            tx_buffer.0.into(), tx_buffer.1.into()
        );
        let mut socket = UdpSocket::<'a, 'b>::new(rx_buffer, tx_buffer);
        socket.bind(endpoint)
            .unwrap_or_else(|_| {});
        let handle = sockets.add(socket);

        Receiver {
            handle,
            current_prio: 255,
            last_packet: Instant::from_secs(0),
        }
    }

    pub fn poll<F>(&mut self, sockets: &mut SocketSet, now: Instant, mut callback: F) -> Result<(), Error>
        where F: FnMut(&[u8])
    {
        let mut socket = sockets.get::<UdpSocket>(self.handle);

        if socket.can_recv() {
            let (packet, _sender) = socket.recv()?;
            if packet.len() < 4 {
                return Err(Error::Illegal);
            }
            let channel = packet[0];
            let command = packet[1];
            let data_len = (usize::from(packet[2]) << 8) | usize::from(packet[3]);
            if data_len + 4 != packet.len() {
                return Err(Error::Illegal);
            }
            let data = &packet[4..];

            match command {
                CMD_SET_PIXEL_COLORS if channel <= self.current_prio => {
                    callback(data);
                    self.current_prio = channel;
                    self.last_packet = now;
                    Ok(())
                },
                CMD_SET_PIXEL_COLORS => {
                    // Ignore higher channel (lower prio)
                    Ok(())
                },
                _ =>
                    Err(Error::Unrecognized),
            }
        } else {
            if self.last_packet + Duration::from_millis(PRIO_TIMEOUT) <= now {
                self.current_prio = 255;
            }

            Ok(())
        }
    }
}
