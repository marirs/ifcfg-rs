mod interfaces;
pub(crate) use interfaces::Interface;

mod error;
mod constants;
mod ffi;
mod flags;

pub(crate) use self::error::InterfacesError;

impl From<InterfacesError> for crate::IfCfgError {
    fn from(err: InterfacesError) -> crate::IfCfgError {
        crate::IfCfgError::InterfacesError(err)
    }
}

impl From<interfaces::Kind> for crate::AddressFamily {
    fn from(kind: interfaces::Kind) -> crate::AddressFamily {
        match kind {
            interfaces::Kind::Ipv4 => crate::AddressFamily::IPv4,
            interfaces::Kind::Ipv6 => crate::AddressFamily::IPv6,
            interfaces::Kind::Link => crate::AddressFamily::Link,
            interfaces::Kind::Packet => crate::AddressFamily::Packet,
            interfaces::Kind::Unknown(v) => crate::AddressFamily::Unknown(v),
        }
    }
}

impl From<interfaces::NextHop> for crate::Hops {
    fn from(next_hop: interfaces::NextHop) -> crate::Hops {
        match next_hop {
            interfaces::NextHop::Broadcast(v) => crate::Hops::Broadcast(v),
            interfaces::NextHop::Destination(v) => crate::Hops::Destination(v),
        }
    }
}

impl From<interfaces::Address> for crate::InterfaceAddress {
    fn from(address: interfaces::Address) -> crate::InterfaceAddress {
        crate::InterfaceAddress {
            address_family: address.kind.into(),
            address: address.addr,
            mask: address.mask,
            hop: address.hop.map(|next_hop| next_hop.into()),
        }
    }
}

impl From<&interfaces::Interface> for crate::IfCfg {
    fn from(interface: &interfaces::Interface) -> crate::IfCfg {
        crate::IfCfg {
            name: interface.name.clone(),
            mac: interface.hardware_addr().unwrap().as_string(),
            addresses: interface.addresses.clone().into_iter().map(|address| address.into()).rev().collect(),
            description: "".to_string()
        }
    }
}
