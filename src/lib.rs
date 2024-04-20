//! # natpmp
//!
//! `natpmp` is a NAT-PMP [IETF RFC 6886](https://tools.ietf.org/html/rfc6886) client library in rust.
//! It is a rust implementation of the c library [natpmp](https://github.com/miniupnp/natpmp).

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::ops::Add;
use std::result;
use std::time::{Duration, Instant};

mod asynchronous;
mod error;
mod ffi;

#[cfg(feature = "tokio")]
mod a_tokio;
#[cfg(feature = "tokio")]
pub use a_tokio::*;

#[cfg(feature = "async-std")]
mod a_std;
#[cfg(feature = "async-std")]
pub use a_std::*;

pub use crate::error::*;
use crate::ffi::*;
pub use asynchronous::*;

/// NAT-PMP mini wait milli-seconds
const NATPMP_MIN_WAIT: u64 = 250;

/// NAT-PMP max retry
const NATPMP_MAX_ATTEMPS: u32 = 9;

/// NAT-PMP server port as defined by rfc6886.
pub const NATPMP_PORT: u16 = 5351;

/// NAT-PMP result.
pub type Result<T> = result::Result<T, Error>;

/// Get default gateway.
///
/// # Errors
/// * [`Error::NATPMP_ERR_CANNOTGETGATEWAY`](enum.Error.html#variant.NATPMP_ERR_CANNOTGETGATEWAY)
///
/// # Examples
/// ```
/// use natpmp::*;
///
/// let r = get_default_gateway();
/// assert_eq!(r.is_ok(), true);
/// ```
pub fn get_default_gateway() -> Result<Ipv4Addr> {
    let mut addr: u32 = 0;
    let result: i32 = unsafe { getdefaultgateway(&mut addr) };
    if result == 0 {
        addr = u32::from_be(addr); // to native order
        return Ok(Ipv4Addr::from(addr));
    }
    Err(Error::NATPMP_ERR_CANNOTGETGATEWAY)
}

/// NAT-PMP mapping protocol.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Protocol {
    UDP,
    TCP,
}

/// NAT-PMP response type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ResponseType {
    Gateway,
    UDP,
    TCP,
}

/// Gateway response.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GatewayResponse {
    epoch: u32,
    public_address: Ipv4Addr,
}

impl GatewayResponse {
    /// Gateway public/external address.
    pub fn public_address(&self) -> &Ipv4Addr {
        &self.public_address
    }

    /// Seconds since epoch.
    ///
    /// **Note: May be not accurate.**
    pub fn epoch(&self) -> u32 {
        self.epoch
    }
}

/// Mapping response.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MappingResponse {
    epoch: u32,
    private_port: u16,
    public_port: u16,
    lifetime: Duration,
}

impl MappingResponse {
    /// Seconds since epoch.
    ///
    /// **Note: May be not accurate.**
    pub fn epoch(&self) -> u32 {
        self.epoch
    }

    /// Private/internal port.
    pub fn private_port(&self) -> u16 {
        self.private_port
    }

    /// Public/external port.
    pub fn public_port(&self) -> u16 {
        self.public_port
    }

    /// Mapping lifetime.
    pub fn lifetime(&self) -> &Duration {
        &self.lifetime
    }
}

/// NAT-PMP response.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Response {
    Gateway(GatewayResponse),
    UDP(MappingResponse),
    TCP(MappingResponse),
}

/// NAT-PMP main struct.
///
/// # Examples
/// ```
/// use std::thread;
/// use std::time::Duration;
/// use natpmp::*;
///
/// # fn main() -> Result<()> {
/// let mut n = Natpmp::new()?;
/// n.send_port_mapping_request(Protocol::UDP, 4020, 4020, 30)?;
/// thread::sleep(Duration::from_millis(100));
/// let response = n.read_response_or_retry()?;
///  match response {
///      Response::UDP(ur) => {
///          assert_eq!(ur.private_port(), 4020);
///          assert_eq!(ur.public_port(), 4020);
///      }
///      _ => panic!("Not a udp mapping response"),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Natpmp {
    s: UdpSocket,
    gateway: Ipv4Addr,
    has_pending_request: bool,
    pending_request: [u8; 12],
    pending_request_len: usize,
    try_number: u32,
    retry_time: Instant,
}

impl Natpmp {
    /// Create a NAT-PMP object with default gateway.
    ///
    /// # Errors
    /// See [`get_default_gateway`](fn.get_default_gateway.html) and [`Natpmp::new_with`](struct.Natpmp.html#method.new_with).
    ///
    /// # Examples
    /// ```
    /// use natpmp::*;
    ///
    /// let n = Natpmp::new();
    /// assert_eq!(n.is_ok(), true);
    /// ```
    pub fn new() -> Result<Natpmp> {
        let gateway = get_default_gateway()?;
        Natpmp::new_with(gateway)
    }

    /// Create a NAT-PMP object with a specified gateway.
    ///
    /// # Errors
    /// * [`Error::NATPMP_ERR_SOCKETERROR`](enum.Error.html#variant.NATPMP_ERR_SOCKETERROR)
    /// * [`Error::NATPMP_ERR_FCNTLERROR`](enum.Error.html#variant.NATPMP_ERR_FCNTLERROR)
    /// * [`Error::NATPMP_ERR_CONNECTERR`](enum.Error.html#variant.NATPMP_ERR_CONNECTERR)
    ///
    /// # Examples
    /// ```
    /// use std::str::FromStr;
    /// use std::net::Ipv4Addr;
    /// use natpmp::*;
    ///
    /// let n = Natpmp::new_with("192.168.0.1".parse().unwrap()).unwrap();
    /// ```
    pub fn new_with(gateway: Ipv4Addr) -> Result<Natpmp> {
        let s: UdpSocket;
        if let Ok(udpsock) = UdpSocket::bind("0.0.0.0:0") {
            s = udpsock;
        } else {
            return Err(Error::NATPMP_ERR_SOCKETERROR);
        }
        if s.set_nonblocking(true).is_err() {
            return Err(Error::NATPMP_ERR_FCNTLERROR);
        }
        let gateway_sockaddr = SocketAddrV4::new(gateway, NATPMP_PORT);
        if s.connect(gateway_sockaddr).is_err() {
            return Err(Error::NATPMP_ERR_CONNECTERR);
        }
        let n = Natpmp {
            s,
            gateway,
            has_pending_request: false,
            pending_request: [0u8; 12],
            pending_request_len: 0,
            try_number: 0,
            retry_time: Instant::now(),
        };
        Ok(n)
    }

    /// NAT-PMP gateway address.
    ///
    /// # Examples
    /// ```
    /// use std::net::Ipv4Addr;
    /// use natpmp::*;
    ///
    /// # fn main() -> Result<()> {
    /// let gateway = Ipv4Addr::from([192, 168, 0, 1]);
    /// let n = Natpmp::new_with(gateway)?;
    /// assert_eq!(n.gateway(), &gateway);
    /// # Ok(())
    /// # }
    /// ```
    pub fn gateway(&self) -> &Ipv4Addr {
        &self.gateway
    }

    fn send_pending_request(&self) -> Result<()> {
        if let Ok(n) = self
            .s
            .send(&self.pending_request[0..self.pending_request_len])
        {
            if n == self.pending_request_len {
                return Ok(());
            }
        }
        Err(Error::NATPMP_ERR_SENDERR)
    }

    fn send_natpmp_request(&mut self) -> Result<()> {
        self.has_pending_request = true;
        self.try_number = 1;
        let result = self.send_pending_request();
        self.retry_time = Instant::now();
        self.retry_time = self.retry_time.add(Duration::from_millis(250));
        result
    }

    /// Get timeout duration of the currently pending NAT-PMP request.
    ///
    /// # Errors:
    /// * [`Error::NATPMP_ERR_NOPENDINGREQ`](enum.Error.html#variant.NATPMP_ERR_NOPENDINGREQ)
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use natpmp::*;
    ///
    /// # fn main() -> Result<()> {
    /// let mut n = Natpmp::new()?;
    /// n.send_public_address_request()?;
    /// // do something
    /// let duration = n.get_natpmp_request_timeout()?;
    /// if duration <= Duration::from_millis(10) {
    ///     // read response ...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_natpmp_request_timeout(&self) -> Result<Duration> {
        if !self.has_pending_request {
            return Err(Error::NATPMP_ERR_NOPENDINGREQ);
        }
        let now = Instant::now();
        if now > self.retry_time {
            return Ok(Duration::from_millis(0));
        }
        let duration = self.retry_time - now;
        Ok(duration)
    }

    /// Send public address request.
    ///
    /// # Errors
    /// * [`Error::NATPMP_ERR_SENDERR`](enum.Error.html#variant.NATPMP_ERR_SENDERR)
    ///
    /// # Examples
    /// ```
    /// use natpmp::*;
    ///
    /// # fn main() -> Result<()> {
    /// let mut n = Natpmp::new()?;
    /// n.send_public_address_request()?;
    /// // do something then read response
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_public_address_request(&mut self) -> Result<()> {
        self.pending_request[0] = 0;
        self.pending_request[1] = 0;
        self.pending_request_len = 2;
        self.send_natpmp_request()
    }

    /// Send new port mapping request.
    ///
    /// # Errors
    /// * [`Error::NATPMP_ERR_SENDERR`](enum.Error.html#variant.NATPMP_ERR_SENDERR)
    ///
    /// # Examples
    /// ```
    /// use natpmp::*;
    ///
    /// # fn main() -> Result<()> {
    /// let mut n = Natpmp::new()?;
    /// n.send_port_mapping_request(Protocol::UDP, 4020, 4020, 30)?;
    /// // do something then read response
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_port_mapping_request(
        &mut self,
        protocol: Protocol,
        private_port: u16,
        public_port: u16,
        lifetime: u32,
    ) -> Result<()> {
        self.pending_request[0] = 0;
        self.pending_request[1] = match protocol {
            Protocol::UDP => 1,
            _ => 2,
        };
        self.pending_request[2] = 0; // reserved
        self.pending_request[3] = 0; // reserved
                                     // private port
        self.pending_request[4] = (private_port >> 8 & 0xff) as u8;
        self.pending_request[5] = (private_port & 0xff) as u8;
        // public port
        self.pending_request[6] = (public_port >> 8 & 0xff) as u8;
        self.pending_request[7] = (public_port & 0xff) as u8;
        // lifetime
        self.pending_request[8] = ((lifetime >> 24) & 0xff) as u8;
        self.pending_request[9] = ((lifetime >> 16) & 0xff) as u8;
        self.pending_request[10] = ((lifetime >> 8) & 0xff) as u8;
        self.pending_request[11] = (lifetime & 0xff) as u8;
        self.pending_request_len = 12;
        self.send_natpmp_request()
    }

    fn read_response(&self) -> Result<Response> {
        let mut buf = [0u8; 16];
        match self.s.recv_from(&mut buf) {
            Err(e) => match e.raw_os_error() {
                Some(code) => {
                    if code == unsafe { RS_EWOULDBLOCK } {
                        return Err(Error::NATPMP_TRYAGAIN);
                    }
                    if code == unsafe { RS_ECONNREFUSED } {
                        return Err(Error::NATPMP_ERR_NOGATEWAYSUPPORT);
                    }
                }
                _ => {
                    return Err(Error::NATPMP_ERR_RECVFROM);
                }
            },
            Ok((_, sockaddr)) => {
                // check gateway address
                if let SocketAddr::V4(s) = sockaddr {
                    if s.ip() != &self.gateway {
                        return Err(Error::NATPMP_ERR_WRONGPACKETSOURCE);
                    }
                }
                // version
                if buf[0] != 0 {
                    return Err(Error::NATPMP_ERR_UNSUPPORTEDVERSION);
                }
                // opcode
                if buf[1] < 128 || buf[1] > 130 {
                    return Err(Error::NATPMP_ERR_UNSUPPORTEDOPCODE);
                }
                // result code
                let resultcode = u16::from_be_bytes([buf[2], buf[3]]);
                if resultcode != 0 {
                    return Err(match resultcode {
                        1 => Error::NATPMP_ERR_UNSUPPORTEDVERSION,
                        2 => Error::NATPMP_ERR_NOTAUTHORIZED,
                        3 => Error::NATPMP_ERR_NETWORKFAILURE,
                        4 => Error::NATPMP_ERR_OUTOFRESOURCES,
                        5 => Error::NATPMP_ERR_UNSUPPORTEDOPCODE,
                        _ => Error::NATPMP_ERR_UNDEFINEDERROR,
                    });
                }
                // epoch
                let epoch = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
                // result
                let rsp_type = buf[1] & 0x7f;
                return Ok(match rsp_type {
                    0 => Response::Gateway(GatewayResponse {
                        epoch,
                        public_address: Ipv4Addr::from(u32::from_be_bytes([
                            buf[8], buf[9], buf[10], buf[11],
                        ])),
                    }),
                    _ => {
                        let private_port = u16::from_be_bytes([buf[8], buf[9]]);
                        let public_port = u16::from_be_bytes([buf[10], buf[11]]);
                        let lifetime = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
                        let lifetime = Duration::from_secs(lifetime.into());
                        let m = MappingResponse {
                            epoch,
                            private_port,
                            public_port,
                            lifetime,
                        };
                        if rsp_type == 1 {
                            Response::UDP(m)
                        } else {
                            Response::TCP(m)
                        }
                    }
                });
            }
        }
        Err(Error::NATPMP_ERR_RECVFROM)
    }

    /// Read NAT-PMP response if possible
    ///
    /// # Errors
    /// * [`Error::NATPMP_TRYAGAIN`](enum.Error.html#variant.NATPMP_TRYAGAIN)
    /// * [`Error::NATPMP_ERR_NOPENDINGREQ`](enum.Error.html#variant.NATPMP_ERR_NOPENDINGREQ)
    /// * [`Error::NATPMP_ERR_NOGATEWAYSUPPORT`](enum.Error.html#variant.NATPMP_ERR_NOGATEWAYSUPPORT)
    /// * [`Error::NATPMP_ERR_RECVFROM`](enum.Error.html#variant.NATPMP_ERR_RECVFROM)
    /// * [`Error::NATPMP_ERR_WRONGPACKETSOURCE`](enum.Error.html#variant.NATPMP_ERR_WRONGPACKETSOURCE)
    /// * [`Error::NATPMP_ERR_UNSUPPORTEDVERSION`](enum.Error.html#variant.NATPMP_ERR_UNSUPPORTEDVERSION)
    /// * [`Error::NATPMP_ERR_UNSUPPORTEDOPCODE`](enum.Error.html#variant.NATPMP_ERR_UNSUPPORTEDOPCODE)
    /// * [`Error::NATPMP_ERR_UNSUPPORTEDVERSION`](enum.Error.html#variant.NATPMP_ERR_UNSUPPORTEDVERSION)
    /// * [`Error::NATPMP_ERR_NOTAUTHORIZED`](enum.Error.html#variant.NATPMP_ERR_NOTAUTHORIZED)
    /// * [`Error::NATPMP_ERR_NETWORKFAILURE`](enum.Error.html#variant.NATPMP_ERR_NETWORKFAILURE)
    /// * [`Error::NATPMP_ERR_OUTOFRESOURCES`](enum.Error.html#variant.NATPMP_ERR_OUTOFRESOURCES)
    /// * [`Error::NATPMP_ERR_UNSUPPORTEDOPCODE`](enum.Error.html#variant.NATPMP_ERR_OUTOFRESOURCES)
    /// * [`Error::NATPMP_ERR_UNDEFINEDERROR`](enum.Error.html#variant.NATPMP_ERR_UNDEFINEDERROR)
    ///
    /// # Examples
    /// ```
    /// use std::thread;
    /// use std::time::Duration;
    /// use natpmp::*;
    ///
    /// # fn main() -> Result<()> {
    /// let mut n = Natpmp::new()?;
    /// n.send_public_address_request()?;
    /// thread::sleep(Duration::from_millis(250));
    /// let response = n.read_response_or_retry()?;
    /// # Ok(())
    /// # }
    ///
    /// ```
    pub fn read_response_or_retry(&mut self) -> Result<Response> {
        if !self.has_pending_request {
            return Err(Error::NATPMP_ERR_NOPENDINGREQ);
        }
        let result = self.read_response();
        if let Err(e) = result {
            match e {
                Error::NATPMP_TRYAGAIN => {
                    let now = Instant::now();
                    // time to retry or not
                    if now >= self.retry_time {
                        if self.try_number >= NATPMP_MAX_ATTEMPS {
                            return Err(Error::NATPMP_ERR_NOGATEWAYSUPPORT);
                        }
                        // double dealy
                        let delay = NATPMP_MIN_WAIT * (1 << self.try_number); // ms
                        self.retry_time = self.retry_time.add(Duration::from_millis(delay)); // next time
                        self.try_number += 1;
                        self.send_pending_request()?;
                    }
                }
                _ => return Err(e),
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_ffi() {
        assert!(get_default_gateway().is_ok());
        assert_ne!(0, unsafe { RS_EWOULDBLOCK });
        assert_ne!(0, unsafe { RS_ECONNREFUSED });
    }

    #[test]
    fn test_natpmp() -> Result<()> {
        assert!(Natpmp::new().is_ok());
        let addr = "192.168.0.1".parse().unwrap();
        let n = Natpmp::new_with(addr)?;
        assert_eq!(*n.gateway(), addr);
        Ok(())
    }

    #[test]
    fn test_get_public_address() -> Result<()> {
        let mut n = Natpmp::new()?;
        n.send_public_address_request()?;
        thread::sleep(Duration::from_millis(250));
        let r = n.read_response_or_retry()?;
        match r {
            Response::Gateway(_) => {}
            _ => panic!("Not a gateway response"),
        }
        Ok(())
    }

    #[test]
    fn test_tcp_mapping() -> Result<()> {
        let mut n = Natpmp::new()?;
        n.send_port_mapping_request(Protocol::TCP, 14020, 14020, 10)?;
        thread::sleep(Duration::from_millis(250));
        let r = n.read_response_or_retry()?;
        match r {
            Response::TCP(tr) => {
                assert_eq!(tr.private_port(), 14020);
                assert_eq!(tr.public_port(), 14020);
            }
            _ => panic!("Not a tcp mapping response"),
        }
        Ok(())
    }

    #[test]
    fn test_udp_mapping() -> Result<()> {
        let mut n = Natpmp::new()?;
        n.send_port_mapping_request(Protocol::UDP, 14020, 14020, 10)?;
        thread::sleep(Duration::from_millis(250));
        let r = n.read_response_or_retry()?;
        match r {
            Response::UDP(ur) => {
                assert_eq!(ur.private_port(), 14020);
                assert_eq!(ur.public_port(), 14020);
            }
            _ => panic!("Not a udp mapping response"),
        }
        Ok(())
    }

    #[test]
    fn test_error() -> Result<()> {
        let mut n = Natpmp::new()?;
        n.send_port_mapping_request(Protocol::UDP, 14020, 14020, 30)?;
        thread::sleep(Duration::from_millis(250));
        n.read_response_or_retry()?;

        n.send_port_mapping_request(Protocol::UDP, 14021, 14020, 10)?;
        thread::sleep(Duration::from_millis(250));
        match n.read_response_or_retry() {
            Ok(Response::UDP(ur)) => {
                assert_ne!(ur.public_port(), 14020);
            }
            Ok(_) => panic!("Not a udp mapping response!"),
            Err(_) => {}
        }
        Ok(())
    }
}
