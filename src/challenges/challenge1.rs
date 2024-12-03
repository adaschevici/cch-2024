use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;
use std::net::AddrParseError;

#[derive(Deserialize)]
struct FromIpToIp {
    from: String,
    key: String,
}

#[derive(Deserialize)]
struct FromDestToIp {
    from: String,
    to: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4 {
    octets: [u8; 4],
}

impl Ipv4 {
    /// Create a new Ipv4 instance from four octets.
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4 {
            octets: [a, b, c, d],
        }
    }

    /// Create an Ipv4 instance from a standard IPv4 string like "192.168.0.1".
    pub fn from_str(ip_str: &str) -> Result<Self, AddrParseError> {
        // We can leverage the standard library's parsing first.
        let addr = ip_str.parse::<std::net::Ipv4Addr>()?;
        Ok(Ipv4 {
            octets: addr.octets(),
        })
    }

    /// Return the octets as a [u8;4].
    pub fn octets(&self) -> [u8; 4] {
        self.octets
    }

    /// Convert the IPv4 address back to a dot-decimal string like "192.168.0.1".
    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        )
    }
    pub fn add(&self, other: &Ipv4) -> Ipv4 {
        Ipv4 {
            octets: [
                self.octets[0].wrapping_add(other.octets[0]),
                self.octets[1].wrapping_add(other.octets[1]),
                self.octets[2].wrapping_add(other.octets[2]),
                self.octets[3].wrapping_add(other.octets[3]),
            ],
        }
    }
    pub fn sub(&self, other: &Ipv4) -> Ipv4 {
        Ipv4 {
            octets: [
                self.octets[0].wrapping_sub(other.octets[0]),
                self.octets[1].wrapping_sub(other.octets[1]),
                self.octets[2].wrapping_sub(other.octets[2]),
                self.octets[3].wrapping_sub(other.octets[3]),
            ],
        }
    }
}

async fn calculate_ipv5_sum(Query(source_dest): Query<FromIpToIp>) -> String {
    let from = Ipv4::from_str(&source_dest.from).unwrap();
    let key = Ipv4::from_str(&source_dest.key).unwrap();

    format!("{}", from.add(&key).to_string())
}

async fn calculate_ipv5_sub(Query(dest_source): Query<FromDestToIp>) -> String {
    let from = Ipv4::from_str(&dest_source.from).unwrap();
    let to = Ipv4::from_str(&dest_source.to).unwrap();

    format!("{}", to.sub(&from).to_string())
}
pub fn router() -> Router {
    Router::new()
        .route("/dest", get(calculate_ipv5_sum))
        .route("/key", get(calculate_ipv5_sub))
}
