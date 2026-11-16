//! Works out the client's IP address for the per-IP run limit (R5.7).
//!
//! `X-Forwarded-For` is believed only when the connection comes from a trusted proxy, such as Caddy on
//! the Compose network. Any other connection is identified by its socket address, so a client cannot
//! choose its own limit bucket by sending a header.

use std::net::{IpAddr, Ipv4Addr};

use axum::http::HeaderMap;

/// The proxies whose forwarding header the server believes. IPv4 networks only.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrustedProxies {
    networks: Vec<Ipv4Network>,
}

/// One IPv4 network, stored as its first address and its prefix length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ipv4Network {
    network: u32,
    prefix: u8,
}

impl Ipv4Network {
    fn contains(self, address: u32) -> bool {
        let mask = if self.prefix == 0 {
            0
        } else {
            u32::MAX << (32 - self.prefix)
        };
        address & mask == self.network & mask
    }
}

impl TrustedProxies {
    /// Reads a comma-separated list such as `127.0.0.1/32,10.0.0.0/8`. A bare address counts as `/32`.
    /// An empty list trusts nothing.
    ///
    /// # Errors
    ///
    /// Returns a plain reason when an entry is not an IPv4 address or network. The reason never echoes
    /// the entry.
    pub fn parse(text: &str) -> Result<Self, &'static str> {
        let networks = text
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(parse_network)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { networks })
    }

    /// Whether a connection from `address` may set forwarding headers. IPv6 peers are never trusted.
    #[must_use]
    pub fn trusts(&self, address: IpAddr) -> bool {
        match address {
            IpAddr::V4(v4) => {
                let bits = u32::from(v4);
                self.networks.iter().any(|network| network.contains(bits))
            }
            IpAddr::V6(_) => false,
        }
    }
}

fn parse_network(entry: &str) -> Result<Ipv4Network, &'static str> {
    let (address_text, prefix_text) = entry.split_once('/').unwrap_or((entry, "32"));
    let address: Ipv4Addr = address_text
        .parse()
        .map_err(|_| "a trusted proxy entry must be an IPv4 address or network")?;
    let prefix: u8 = prefix_text
        .parse()
        .ok()
        .filter(|prefix| *prefix <= 32)
        .ok_or("a trusted proxy prefix length must be from 0 to 32")?;
    Ok(Ipv4Network {
        network: u32::from(address),
        prefix,
    })
}

/// The client address for per-IP limits.
///
/// From a trusted peer, the rightmost `X-Forwarded-For` address that is not itself a trusted proxy is
/// used. Anything unreadable falls back to the peer address.
#[must_use]
pub fn client_ip(peer: IpAddr, headers: &HeaderMap, trusted: &TrustedProxies) -> IpAddr {
    if !trusted.trusts(peer) {
        return peer;
    }
    let Some(text) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
    else {
        return peer;
    };
    let hops: Option<Vec<IpAddr>> = text.split(',').map(|hop| hop.trim().parse().ok()).collect();
    let Some(hops) = hops else {
        return peer;
    };
    hops.iter()
        .rev()
        .find(|hop| !trusted.trusts(**hop))
        .or_else(|| hops.first())
        .copied()
        .unwrap_or(peer)
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderName, HeaderValue};

    use super::*;

    fn forwarded(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-forwarded-for"),
            HeaderValue::from_static(value),
        );
        headers
    }

    fn ip(text: &str) -> IpAddr {
        text.parse().expect("test address is valid")
    }

    #[test]
    fn parse_accepts_addresses_and_networks() {
        let trusted = TrustedProxies::parse("127.0.0.1/32, 10.0.0.0/8").expect("valid list");
        assert!(trusted.trusts(ip("127.0.0.1")));
        assert!(trusted.trusts(ip("10.200.1.9")));
        assert!(!trusted.trusts(ip("11.0.0.1")));
    }

    #[test]
    fn parse_rejects_bad_entries_without_echoing_them() {
        assert!(TrustedProxies::parse("not-an-ip").is_err());
        assert!(TrustedProxies::parse("10.0.0.0/33").is_err());
    }

    #[test]
    fn empty_list_trusts_nothing() {
        let trusted = TrustedProxies::parse("").expect("empty list is valid");
        assert!(!trusted.trusts(ip("127.0.0.1")));
    }

    #[test]
    fn untrusted_peer_ignores_the_forwarding_header() {
        let trusted = TrustedProxies::parse("10.0.0.0/8").expect("valid list");
        let client = client_ip(ip("127.0.0.1"), &forwarded("203.0.113.7"), &trusted);
        assert_eq!(client, ip("127.0.0.1"));
    }

    #[test]
    fn trusted_peer_uses_the_rightmost_untrusted_hop() {
        let trusted = TrustedProxies::parse("10.0.0.0/8").expect("valid list");
        let client = client_ip(
            ip("10.0.0.1"),
            &forwarded("198.51.100.4, 203.0.113.7, 10.0.0.2"),
            &trusted,
        );
        assert_eq!(client, ip("203.0.113.7"));
    }

    #[test]
    fn unreadable_header_falls_back_to_the_peer() {
        let trusted = TrustedProxies::parse("10.0.0.0/8").expect("valid list");
        let client = client_ip(ip("10.0.0.1"), &forwarded("not an address"), &trusted);
        assert_eq!(client, ip("10.0.0.1"));
    }
}
