use std::io;
use std::net::Ipv4Addr;
use std::time::Duration;

use async_trait::async_trait;

use crate::{
    Error, GatewayResponse, MappingResponse, Protocol, Response, Result, NATPMP_MAX_ATTEMPS,
};

/// A wrapper trait for async udpsocket.
#[async_trait]
pub trait AsyncUdpSocket {
    async fn connect(&self, addr: &str) -> io::Result<()>;

    async fn send(&self, buf: &[u8]) -> io::Result<usize>;

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize>;
}

/// NAT-PMP async client
pub struct NatpmpAsync<S>
where
    S: AsyncUdpSocket,
{
    s: S,
    gateway: Ipv4Addr,
}

/// Create a NAT-PMP object with async udpsocket and gateway
pub fn new_natpmp_async_with<S>(s: S, gateway: Ipv4Addr) -> NatpmpAsync<S>
where
    S: AsyncUdpSocket,
{
    NatpmpAsync { s, gateway }
}

impl<S> NatpmpAsync<S>
where
    S: AsyncUdpSocket,
{
    /// NAT-PMP gateway address.
    pub fn gateway(&self) -> &Ipv4Addr {
        &self.gateway
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
    /// let mut n = new_tokio_natpmp().await?;
    /// n.send_public_address_request().await?;
    /// ```
    pub async fn send_public_address_request(&mut self) -> Result<()> {
        let request = [0_u8; 2];
        let n = self
            .s
            .send(&request[..])
            .await
            .map_err(|_| Error::NATPMP_ERR_SENDERR)?;
        if n != request.len() {
            return Err(Error::NATPMP_ERR_SENDERR);
        }
        Ok(())
    }

    /// Send port mapping request.
    ///
    /// # Errors
    /// * [`Error::NATPMP_ERR_SENDERR`](enum.Error.html#variant.NATPMP_ERR_SENDERR)
    ///
    /// # Examples
    /// ```
    /// use natpmp::*;
    ///
    /// let mut n = new_tokio_natpmp().await?;
    /// n.send_port_mapping_request(Protocol::UDP, 4020, 4020, 30).await?;
    /// ```
    pub async fn send_port_mapping_request(
        &self,
        protocol: Protocol,
        private_port: u16,
        public_port: u16,
        lifetime: u32,
    ) -> Result<()> {
        let mut request = [0_u8; 12];
        request[1] = match protocol {
            Protocol::UDP => 1,
            _ => 2,
        };
        request[2] = 0; // reserved
        request[3] = 0;
        // private port
        request[4] = (private_port >> 8 & 0xff) as u8;
        request[5] = (private_port & 0xff) as u8;
        // public port
        request[6] = (public_port >> 8 & 0xff) as u8;
        request[7] = (public_port & 0xff) as u8;
        // lifetime
        request[8] = ((lifetime >> 24) & 0xff) as u8;
        request[9] = ((lifetime >> 16) & 0xff) as u8;
        request[10] = ((lifetime >> 8) & 0xff) as u8;
        request[11] = (lifetime & 0xff) as u8;

        let n = self
            .s
            .send(&request[..])
            .await
            .map_err(|_| Error::NATPMP_ERR_SENDERR)?;
        if n != request.len() {
            return Err(Error::NATPMP_ERR_SENDERR);
        }
        Ok(())
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
    /// use natpmp::*;
    ///
    /// let mut n = new_tokio_natpmp().await?;
    /// n.send_public_address_request().await?;
    /// let response = n.read_response_or_retry().await?;
    ///
    /// ```
    pub async fn read_response_or_retry(&self) -> Result<Response> {
        let mut buf = [0_u8; 16];
        let mut retries = 0;
        while retries < NATPMP_MAX_ATTEMPS {
            match self.s.recv(&mut buf).await {
                Err(_) => retries += 1,
                Ok(_) => {
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
                    // result
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
        }

        Err(Error::NATPMP_ERR_RECVFROM)
    }
}
