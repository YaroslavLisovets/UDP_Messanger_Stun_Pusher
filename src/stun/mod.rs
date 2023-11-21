
use std::convert::TryInto;
use std::net::{Ipv4Addr,  UdpSocket};
use std::time::Duration;
use rand::Rng;
#[derive(Debug)]
pub enum StunError {
    BindError,
    SendError,
    TimeoutError,
    ReceiveError,
    TransactionIdMismatch,
    UnexpectedResponse,
    GeneralError,
}
pub fn resolve_udp_addr_v4(stun_server_addr: Option<&str>) -> Result<(Ipv4Addr, u16, UdpSocket), StunError> {

    let addr = stun_server_addr.unwrap_or("stun.l.google.com:19302");
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 55165)).map_err(|_| StunError::BindError)?;
    let mut request: [u8; 20] = [0; 20];
    request[1] = 1;
    rand::thread_rng().fill(&mut request[4..]);
    socket.send_to(&request, addr).map_err(|_| StunError::SendError)?;
    socket.set_read_timeout(Some(Duration::new(5, 0))).map_err(|_| StunError::TimeoutError)?;

    let mut buff = [0u8; 32];
    socket.recv(&mut buff).map_err(|_| StunError::ReceiveError)?;

    let message_type = u16::from_be_bytes([buff[0], buff[1]]);
    let transaction_id_bytes: [u8; 16] = buff[4..20].try_into().map_err(|_| StunError::TransactionIdMismatch)?;

    if &request[4..] != &transaction_id_bytes {
        return Err(StunError::TransactionIdMismatch);
    }

    let port = u16::from_be_bytes([buff[26], buff[27]]);
    let ip_bytes: [u8; 4] = match buff[28..32].try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(StunError::UnexpectedResponse),
    };

    match message_type {
        0x0101 => {
            let addr = Ipv4Addr::from(ip_bytes);
            Ok((addr, port, socket))
        }
        0x0020 => Err(StunError::UnexpectedResponse),
        _ => Err(StunError::GeneralError),
    }
}

