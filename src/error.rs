use std::fmt;

/// NAT-PMP error.
///
/// # Note
///
/// These errors are for compatibility only:
/// * [`Error::NATPMP_ERR_INVALIDARGS`](enum.Error.html#variant.NATPMP_ERR_INVALIDARGS)
/// * [`Error::NATPMP_ERR_CLOSEERR`](enum.Error.html#variant.NATPMP_ERR_CLOSEERR)
/// * [`Error::NATPMP_ERR_GETTIMEOFDAYERR`](enum.Error.html#variant.NATPMP_ERR_GETTIMEOFDAYERR)
///
/// # Examples
/// ```
/// use natpmp::*;
///
/// let err = Error::NATPMP_ERR_CANNOTGETGATEWAY;
/// println!("{}", err);
/// ```
///
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    /// Invalid arguments
    NATPMP_ERR_INVALIDARGS,

    /// Failed to create a socket
    NATPMP_ERR_SOCKETERROR,

    /// Can not get default gateway address
    NATPMP_ERR_CANNOTGETGATEWAY,

    /// Failed to close socket
    NATPMP_ERR_CLOSEERR,

    /// Failed to recvfrom socket
    NATPMP_ERR_RECVFROM,

    /// No pending request
    NATPMP_ERR_NOPENDINGREQ,

    /// Gateway does not support NAT-PMP
    NATPMP_ERR_NOGATEWAYSUPPORT,

    /// Failed to connect to the gateway
    NATPMP_ERR_CONNECTERR,

    /// Packet not received from the gateway
    NATPMP_ERR_WRONGPACKETSOURCE,

    /// Failed to send
    NATPMP_ERR_SENDERR,

    /// Failed to set nonblocking
    NATPMP_ERR_FCNTLERROR,

    /// Failed to get time
    NATPMP_ERR_GETTIMEOFDAYERR,

    /// Unsupported NAT-PMP version
    NATPMP_ERR_UNSUPPORTEDVERSION,

    /// Unsupported NAT-PMP opcode
    NATPMP_ERR_UNSUPPORTEDOPCODE,

    /// Unknown NAT-PMP error
    NATPMP_ERR_UNDEFINEDERROR,

    /// Not authorized
    NATPMP_ERR_NOTAUTHORIZED,

    /// Network failure
    NATPMP_ERR_NETWORKFAILURE,

    /// NAT-PMP out of resources
    NATPMP_ERR_OUTOFRESOURCES,

    /// Try again
    NATPMP_TRYAGAIN,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NATPMP_ERR_INVALIDARGS => write!(f, "invalid arguments"),
            Error::NATPMP_ERR_SOCKETERROR => write!(f, "socket failed"),
            Error::NATPMP_ERR_CANNOTGETGATEWAY => {
                write!(f, "cannot get default gateway ip address")
            }
            Error::NATPMP_ERR_CLOSEERR => write!(f, "close failed"),
            Error::NATPMP_ERR_RECVFROM => write!(f, "recvfrom failed"),
            Error::NATPMP_ERR_NOPENDINGREQ => write!(f, "no pending request"),
            Error::NATPMP_ERR_NOGATEWAYSUPPORT => write!(f, "the gateway does not support nat-pmp"),
            Error::NATPMP_ERR_CONNECTERR => write!(f, "connect failed"),
            Error::NATPMP_ERR_WRONGPACKETSOURCE => {
                write!(f, "packet not received from the gateway")
            }
            Error::NATPMP_ERR_SENDERR => write!(f, "send failed"),
            Error::NATPMP_ERR_FCNTLERROR => write!(f, "fcntl failed"),
            Error::NATPMP_ERR_GETTIMEOFDAYERR => write!(f, "get time failed"),
            Error::NATPMP_ERR_UNSUPPORTEDVERSION => {
                write!(f, "unsupported nat-pmp version error from server")
            }
            Error::NATPMP_ERR_UNSUPPORTEDOPCODE => {
                write!(f, "unsupported nat-pmp opcode error from server")
            }
            Error::NATPMP_ERR_UNDEFINEDERROR => write!(f, "undefined nat-pmp server error"),
            Error::NATPMP_ERR_NOTAUTHORIZED => write!(f, "not authorized"),
            Error::NATPMP_ERR_NETWORKFAILURE => write!(f, "network failure"),
            Error::NATPMP_ERR_OUTOFRESOURCES => write!(f, "nat-pmp server out of resources"),
            Error::NATPMP_TRYAGAIN => write!(f, "try again"),
        }
    }
}

impl std::error::Error for Error {}
