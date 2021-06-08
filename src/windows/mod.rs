use std::{
    io::{Result, Error, Write},
    net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr},
    slice::from_raw_parts,
    ffi::CStr,
    ptr,
    mem,
};

use winapi::{
    ctypes::{c_int, wchar_t},
    shared::{
        basetsd::ULONG64,
        minwindef::{ULONG, DWORD},
        winerror::ERROR_SUCCESS,
        ws2def::{AF_INET, AF_INET6, SOCKADDR},
        ws2ipdef::SOCKADDR_IN6_LH,
    },
    um::iptypes::{
        GAA_FLAG_INCLUDE_GATEWAYS
    },
};

use libc::{c_char, c_void, free, malloc, size_t};

const WORKING_BUFFER_SIZEL: size_t = 15000;
const MAX_ADAPTER_ADDRESS_LENGTH: usize = 8;

#[repr(C)]
struct LengthIfIndex {
    length: ULONG,
    ifindex: DWORD,
}

#[repr(C)]
pub struct LengthFlags {
    length: ULONG,
    flags: DWORD,
}

#[repr(C)]
pub struct LengthReserved {
    length: ULONG,
    reserved: DWORD,
}

#[repr(C)]
pub struct SocketAddress {
    lp_sockaddr: *mut SOCKADDR,
    i_sockaddr_length: c_int,
}

#[repr(C)]
pub struct IpAdapterPrefix {
    aol: LengthIfIndex,
    next: *mut IpAdapterPrefix,
    address: SocketAddress,
    prefix_length: ULONG,
}

#[repr(C)]
pub struct IpAdapterUnicastAddress {
    aol: LengthFlags,
    next: *mut IpAdapterUnicastAddress,
    address: SocketAddress,
    prefix_origin: c_int,
    suffix_origin: c_int,
    dad_state: c_int,
    valid_lifetime: ULONG,
    preferred_lifetime: ULONG,
    lease_lifetime: ULONG,
    on_link_prefix_length: u8,
}

#[repr(C)]
pub struct IpAdapterWinsServerAddress {
    aol: LengthReserved,
    next: *mut IpAdapterWinsServerAddress,
    address: SocketAddress,
}

#[repr(C)]
pub struct IpAdapterGatewayAddress {
    aol: LengthReserved,
    next: *mut IpAdapterGatewayAddress,
    address: SocketAddress,
}

#[repr(C)]
pub struct IpAdapterAddresses {
    aol: LengthIfIndex,
    next: *mut IpAdapterAddresses,
    adapter_name: *mut c_char,
    first_unicast_address: *mut IpAdapterUnicastAddress,
    first_anycast_address: *const c_void,
    first_multicast_address: *const c_void,
    first_dns_server_address: *const c_void,
    dns_suffix: *mut wchar_t,
    description: *mut wchar_t,
    friendly_name: *mut wchar_t,
    physical_address: [u8; MAX_ADAPTER_ADDRESS_LENGTH],
    physical_address_length: DWORD,
    flags: DWORD,
    mtu: DWORD,
    if_type: DWORD,
    oper_status: c_int,
    ipv6_if_index: DWORD,
    zone_indices: [DWORD; 16],
    first_prefix: *mut IpAdapterPrefix,
    transmit_link_speed: ULONG64,
    receive_link_speed: ULONG64,
    first_wins_server_address: *mut IpAdapterWinsServerAddress,
    first_gateway_address: *mut IpAdapterGatewayAddress,
}

impl IpAdapterAddresses {
    pub fn name(&self) -> String {
        c_char_array_to_string(self.adapter_name)
    }

    pub fn friendly_name(&self) -> String {
        u16_array_to_string(self.friendly_name)
    }

    pub fn mac_address(&self) -> String {
        physical_address_to_string(
            self.physical_address,
            self.physical_address_length
        )
    }

    pub fn description(&self) -> String {
        u16_array_to_string(self.description)
    }

    pub fn dns_suffix(&self) -> String {
        u16_array_to_string(self.dns_suffix)
    }

    pub fn prefixes(&self) -> PrefixesIterator {
        PrefixesIterator {
            _head: self,
            next: self.first_prefix,
        }
    }

    pub fn unicast_addresses(&self) -> UnicastAddressesIterator {
        UnicastAddressesIterator {
            _head: self,
            next: self.first_unicast_address,
        }
    }
}

#[link(name = "iphlpapi")]
extern "system" {
    fn GetAdaptersAddresses(
        family: ULONG,
        flags: ULONG,
        reserved: *const c_void,
        addresses: *mut IpAdapterAddresses,
        size: *mut ULONG,
    ) -> ULONG;
}

pub struct Adapters {
    inner: *const IpAdapterAddresses,
}

impl Adapters {
    #[allow(unsafe_code)]
    pub fn new() -> Result<Self> {
        let mut buffersize: ULONG = WORKING_BUFFER_SIZEL as ULONG;
        let mut p_adapter: *mut IpAdapterAddresses;
        // let mut buffersize: c_ulong = 15000;
        // let mut ifaddrs: *mut IpAdapterAddresses;

        loop {
            unsafe {
                p_adapter = malloc(buffersize as size_t) as *mut IpAdapterAddresses;
                if p_adapter.is_null() {
                    panic!("Failed to allocate buffer in IfAddrs()");
                }

                let retcode = GetAdaptersAddresses(
                    0,
                    GAA_FLAG_INCLUDE_GATEWAYS as ULONG,
                    ptr::null(),
                    p_adapter,
                    &mut buffersize,
                );

                match retcode {
                    ERROR_SUCCESS => break,
                    111 => {
                        free(p_adapter as *mut c_void);
                        buffersize *= 2;
                        continue;
                    }
                    _ => return Err(Error::last_os_error()),
                }
            }
        }

        Ok(Self { inner: p_adapter })
    }

    pub fn iter(&self) -> IfAddrsIterator {
        IfAddrsIterator {
            _head: self,
            next: self.inner,
        }
    }
}

impl Drop for Adapters {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            free(self.inner as *mut c_void);
        }
    }
}

pub struct IfAddrsIterator<'a> {
    _head: &'a Adapters,
    next: *const IpAdapterAddresses,
}

impl<'a> Iterator for IfAddrsIterator<'a> {
    type Item = &'a IpAdapterAddresses;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).next;

            result
        })
    }
}

pub struct PrefixesIterator<'a> {
    _head: &'a IpAdapterAddresses,
    next: *const IpAdapterPrefix,
}

impl<'a> Iterator for PrefixesIterator<'a> {
    type Item = &'a IpAdapterPrefix;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).next;

            result
        })
    }
}

pub struct UnicastAddressesIterator<'a> {
    _head: &'a IpAdapterAddresses,
    next: *const IpAdapterUnicastAddress,
}

impl<'a> Iterator for UnicastAddressesIterator<'a> {
    type Item = &'a IpAdapterUnicastAddress;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).next;

            result
        })
    }
}

fn u16_array_to_string(p: *const u16) -> String {
    use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
    unsafe {
        if p.is_null() {
            return String::new();
        }
        let mut amt = 0usize;
        while !p.add(amt).is_null() && *p.add(amt) != 0u16 {
            amt += 1;
        }
        let u16s = from_raw_parts(p, amt);
        decode_utf16(u16s.iter().cloned())
            .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
            .collect::<String>()
    }
}

fn c_char_array_to_string(p: *const c_char) -> String {
    unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() }
}

fn physical_address_to_string(array: [u8; 8], length: DWORD) -> String {
    let mut bytes = Vec::with_capacity(length as usize);
    for (idx, b) in array.iter().enumerate().take(length as usize) {
        if idx == 0 {
            write!(&mut bytes, "{:02X}", b).unwrap();
        } else {
            write!(&mut bytes, "-{:02X}", b).unwrap();
        }
    }
    String::from_utf8_lossy(&bytes[..]).into_owned()
}

// This faster than [u8;4], but v6 is slower if use this..
// And the scan() method is slower also.
fn netmask_v4(bits: u8) -> Ipv4Addr {
    let mut i = (0..4).map(|idx| {
        let idx8 = idx << 3;
        match (bits as usize > idx8, bits as usize > idx8 + 8) {
            (true, true) => 255,
            (true, false) => 255u8.wrapping_shl((8 - bits % 8) as u32),
            _ => 0,
        }
    });
    Ipv4Addr::new(
        i.next().unwrap(),
        i.next().unwrap(),
        i.next().unwrap(),
        i.next().unwrap(),
    )
}

fn netmask_v6(bits: u8) -> Ipv6Addr {
    let mut tmp = [0u16; 8];
    (0..8).for_each(|idx| {
        let idx16 = idx << 4;
        match (bits as usize > idx16, bits as usize > idx16 + 16) {
            (true, true) => {
                tmp[idx] = 0xffff;
            }
            (true, false) => {
                tmp[idx] = 0xffffu16.wrapping_shl((16 - bits % 16) as u32);
            }
            _ => {}
        }
    });
    Ipv6Addr::new(
        tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5], tmp[6], tmp[7],
    )
}

fn sockaddr_to_net_socket_addr(sockaddr: *mut SOCKADDR) -> Option<SocketAddr> {
    let s_addr = unsafe { *sockaddr };
    match sockaddr.into() {
        crate::AddressFamily::IPv4 => {
            let s_addr =IpAddr::V4(Ipv4Addr::new(
                s_addr.sa_data[2] as u8,
                s_addr.sa_data[3] as u8,
                s_addr.sa_data[4] as u8,
                s_addr.sa_data[5] as u8,
            ));
            return Some(SocketAddr::new(s_addr, 0))
        },
        crate::AddressFamily::IPv6 => {
            #[allow(clippy::cast_ptr_alignment)]
                let addr6: *const SOCKADDR_IN6_LH = sockaddr as *const SOCKADDR_IN6_LH;
            let mut a: [u8; 16] = unsafe { *std::ptr::read_unaligned(addr6).sin6_addr.u.Byte() };
            a[..].reverse();
            let a: [u16; 8] = unsafe { mem::transmute(a) };
            let addr = IpAddr::V6(Ipv6Addr::new(
                a[7], a[6], a[5], a[4], a[3], a[2], a[1], a[0],
            ));
            Some(SocketAddr::new(addr, 0))
        },
        _ => None,
    }
}

fn get_netmask(bits: u8) -> Option<SocketAddr> {
    let netmask = if bits <=32 {
        Some(IpAddr::V4(netmask_v4(bits)))
    } else if bits <= 128 {
        Some(IpAddr::V6(netmask_v6(bits)))
    } else {
        None
    };
    match netmask {
        Some(n) => Some(SocketAddr::new(n, 0)),
        None => None
    }
}

impl From<*mut SOCKADDR> for crate::AddressFamily {
    fn from(sockaddr: *mut SOCKADDR) -> crate::AddressFamily {
        match unsafe { *sockaddr }.sa_family as i32 {
            AF_INET => crate::AddressFamily::IPv4,
            AF_INET6 => crate::AddressFamily::IPv6,
            value => crate::AddressFamily::Unknown(value),
        }
    }
}

impl From<&IpAdapterUnicastAddress> for crate::InterfaceAddress {
    fn from(address: &IpAdapterUnicastAddress) -> crate::InterfaceAddress {
        crate::InterfaceAddress {
            address_family: address.address.lp_sockaddr.into(),
            address: sockaddr_to_net_socket_addr(address.address.lp_sockaddr),
            mask: get_netmask(address.on_link_prefix_length.clone()),
            hop: None,
        }
    }
}

impl From<&IpAdapterAddresses> for crate::IfCfg {
    fn from(adapter: &IpAdapterAddresses) -> crate::IfCfg {
        crate::IfCfg {
            name: adapter.friendly_name(),
            mac: adapter.mac_address(),
            addresses: adapter.unicast_addresses()
                .map(|addr| addr.into())
                .map(|mut addr: crate::InterfaceAddress| {
                    let first_gw_addr = adapter.first_gateway_address.clone();
                    if !first_gw_addr.is_null() {
                        let gw_sockaddr = unsafe{(*first_gw_addr).address.lp_sockaddr};
                        let gw_addr = sockaddr_to_net_socket_addr(gw_sockaddr).unwrap();
                        match gw_addr.ip() {
                            IpAddr::V4(ipv4) => {
                                if ipv4.is_broadcast() {
                                    addr.hop = Some(crate::Hops::Broadcast(gw_addr));
                                } else {
                                    addr.hop = Some(crate::Hops::Destination(gw_addr));
                                }
                            },
                            _ => {
                                addr.hop = Some(crate::Hops::Destination(gw_addr));
                            }
                        }
                    }
                    addr
                })
                .collect(),
            description: adapter.description().clone()
        }
    }
}
