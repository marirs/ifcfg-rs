#[cfg(unix)]
#[macro_use]
extern crate bitflags;
#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

use std::{
    fmt,
    net,
    io,
};
use serde::Serialize;
use serde_json;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[derive(Debug)]
pub enum IfCfgError {
    #[cfg(unix)]
    InterfacesError(unix::InterfacesError),
    IOError(io::Error),
    SerdeJsonError(serde_json::error::Error),
    Generic(&'static str),
}

impl fmt::Display for IfCfgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            #[cfg(unix)]
            IfCfgError::InterfacesError(err) => write!(f, "InterfacesError({})", err),
            IfCfgError::IOError(err) => write!(f, "IOError({})", err),
            IfCfgError::SerdeJsonError(err) => write!(f, "SerdeJsonError({})", err),
            IfCfgError::Generic(err) => write!(f, "Generic({})", err),
        }
    }
}

impl From<io::Error> for IfCfgError {
    fn from(err: io::Error) -> IfCfgError {
        IfCfgError::IOError(err)
    }
}

impl From<serde_json::error::Error> for IfCfgError {
    fn from(err: serde_json::error::Error) -> IfCfgError {
        IfCfgError::SerdeJsonError(err)
    }
}

pub type Result<T> = std::result::Result<T, IfCfgError>;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum AddressFamily {
    IPv4,
    IPv6,
    Link,
    Packet,
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Hops {
    Broadcast(net::SocketAddr),
    Destination(net::SocketAddr),
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct InterfaceAddress {
    pub address_family: AddressFamily,
    pub address: Option<net::SocketAddr>,
    pub mask: Option<net::SocketAddr>,
    pub hop: Option<Hops>,
}

#[derive(Debug, Serialize)]
pub struct IfCfg {
    pub name: String,
    pub mac: String,
    pub addresses: Vec<InterfaceAddress>,
    pub description: String,
}

impl IfCfg {
    pub fn get() -> Result<Vec<IfCfg>> {
        //! Gets the Interfaces from a computer
        //!
        //! ## Example
        //! ```rust
        //! use ifcfg::{IfCfg, Result};
        //!
        //! fn main() -> Result<()> {
        //!     let adapters = IfCfg::get().expect("could not get interfaces");
        //!     println!("{}", serde_json::to_string(&adapters)?);
        //!     Ok(())
        //! }
        //! ```
        let mut results = Vec::<IfCfg>::new();

        #[cfg(unix)]
        {
            let interfaces = unix::Interface::get_all()?;
            for interface in interfaces.iter() {
                results.push(interface.into());
            }
        }
        #[cfg(windows)]
        {
            let adapters = windows::Adapters::new()?;
            for adapter in adapters.iter() {
                results.push(adapter.into());
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_adapters () {
        let x = IfCfg::get();
        assert!(x.is_ok());
    }
}