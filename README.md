IFCFG
-----
![Crates.io](https://img.shields.io/crates/v/ifcfg)
[![Documentation](https://docs.rs/ifcfg/badge.svg)](https://docs.rs/ifcfg/0.1.0/ifcfg/struct.IfCfg.html)
[![Build Status](https://travis-ci.com/marirs/ifcfg-rs.svg?branch=master)](https://travis-ci.com/marirs/ifcfg-rs)

IFCFG (ifconfig) is a Rust library to get network interfaces information for 
Windows/Linux/Mac

#### Requirements
- Rust

#### Include in project
```toml
[dependencies]
ifcfg = "0.1.1"
```

#### Example
```rust
use ifcfg;

fn main() -> ifcfg::Result<()> {
    let ifaces = ifcfg::IfCfg::get().expect("could not get interfaces");
    println!("{:#?}", &ifaces);
    Ok(())
}
```

---
#### Compile
```bash
cargo b
```

#### Tests
```bash
cargo t
```

#### Run the included example
````bash
cargo run --example interfaces
   Compiling ifcfg v0.1.0 (/root/ifcfg)
    Finished dev [optimized + debuginfo] target(s) in 0.63s
     Running `target/debug/examples/interfaces`
[
    IfCfg {
        name: "lo",
        mac: "00:00:00:00:00:00",
        addresses: [
            InterfaceAddress {
                address_family: IPv6,
                address: Some(
                    [::1]:0,
                ),
                mask: Some(
                    [ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff]:0,
                ),
                hop: None,
            },
            InterfaceAddress {
                address_family: IPv4,
                address: Some(
                    127.0.0.1:0,
                ),
                mask: Some(
                    255.0.0.0:0,
                ),
                hop: Some(
                    Destination(
                        127.0.0.1:0,
                    ),
                ),
            },
            InterfaceAddress {
                address_family: Packet,
                address: None,
                mask: None,
                hop: None,
            },
        ],
        description: "",
    },
    IfCfg {
        name: "ens33",
        mac: "00:0c:29:0a:e0:b4",
        addresses: [
            InterfaceAddress {
                address_family: IPv6,
                address: Some(
                    [fe80::20c:29ff:fe0a:e0b4%2]:0,
                ),
                mask: Some(
                    [ffff:ffff:ffff:ffff::]:0,
                ),
                hop: None,
            },
            InterfaceAddress {
                address_family: IPv6,
                address: Some(
                    [fd15:4ba5:5a2b:1008:20c:29ff:fe0a:e0b4]:0,
                ),
                mask: Some(
                    [ffff:ffff:ffff:ffff::]:0,
                ),
                hop: None,
            },
            InterfaceAddress {
                address_family: IPv4,
                address: Some(
                    192.168.2.3:0,
                ),
                mask: Some(
                    255.255.255.0:0,
                ),
                hop: Some(
                    Broadcast(
                        192.168.2.255:0,
                    ),
                ),
            },
            InterfaceAddress {
                address_family: Packet,
                address: None,
                mask: None,
                hop: None,
            },
        ],
        description: "",
    },
]

````

---
License: MIT/Apache
