// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # private key
//!  A private key represents the secret data associated with its proposed use
//!
use std::str::FromStr;
use util::Error;
use secp256k1::Secp256k1;
use secp256k1::key::{PublicKey, SecretKey};
use util::address::Address;
use network::constants::Network;
use util::base58;

#[derive(Clone, PartialEq, Eq)]
/// A Bitcoin ECDSA private key
pub struct Privkey {
    /// Whether this private key represents a compressed address
    pub compressed: bool,
    /// The network on which this key should be used
    pub network: Network,
    /// The actual ECDSA key
    pub key: SecretKey
}

impl Privkey {
    /// Creates an address from a public key
    #[inline]
    pub fn from_secret_key(key: SecretKey, compressed: bool, network: Network) -> Privkey {
        Privkey {
            compressed: compressed,
            network: network,
            key: key,
        }
    }

    /// Computes the public key as supposed to be used with this secret
    pub fn public_key(&self, secp: &Secp256k1) -> Result<PublicKey, Error> {
        Ok(PublicKey::from_secret_key(secp, &self.key)?)
    }

    /// Converts a private key to a segwit address
    #[inline]
    pub fn to_address(&self, secp: &Secp256k1) -> Result<Address, Error> {
        Ok(Address::p2wpkh(&self.public_key(secp)?, self.network))
    }

    /// Converts a private key to a legacy (non-segwit) address
    #[inline]
    pub fn to_legacy_address(&self, secp: &Secp256k1) -> Result<Address, Error> {
        if self.compressed {
            Ok(Address::p2pkh(&self.public_key(secp)?, self.network))
        }
        else {
            Ok(Address::p2upkh(&self.public_key(secp)?, self.network))
        }
    }

    /// Accessor for the underlying secp key
    #[inline]
    pub fn secret_key(&self) -> &SecretKey {
        &self.key
    }

    /// Accessor for the underlying secp key that consumes the privkey
    #[inline]
    pub fn into_secret_key(self) -> SecretKey {
        self.key
    }

    /// Accessor for the network type
    #[inline]
    pub fn network(&self) -> Network {
        self.network
    }

    /// Accessor for the compressed flag
    #[inline]
    pub fn is_compressed(&self) -> bool {
        self.compressed
    }
}

impl ToString for Privkey {
    fn to_string(&self) -> String {
        let mut ret = [0; 34];
        ret[0] = match self.network {
            Network::Bitcoin => 128,
            Network::Testnet | Network::Regtest => 239,
        };
        ret[1..33].copy_from_slice(&self.key[..]);
        if self.compressed {
            ret[33] = 1;
            base58::check_encode_slice(&ret[..])
        } else {
            base58::check_encode_slice(&ret[..33])
        }
    }
}

impl FromStr for Privkey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Privkey, Error> {
        let data = try!(base58::from_check(s));

        let compressed = match data.len() {
            33 => false,
            34 => true,
            _ => { return Err(Error::Base58(base58::Error::InvalidLength(data.len()))); }
        };

        let network = match data[0] {
            128 => Network::Bitcoin,
            239 => Network::Testnet,
            x   => { return Err(Error::Base58(base58::Error::InvalidVersion(vec![x]))); }
        };

        let secp = Secp256k1::without_caps();
        let key = try!(SecretKey::from_slice(&secp, &data[1..33])
            .map_err(|_| base58::Error::Other("Secret key out of range".to_owned())));

        Ok(Privkey {
            compressed: compressed,
            network: network,
            key: key
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Privkey;
    use secp256k1::Secp256k1;
    use std::str::FromStr;
    use network::constants::Network::Testnet;
    use network::constants::Network::Bitcoin;

    #[test]
    fn test_key_derivation() {
        // testnet compressed
        let sk = Privkey::from_str("cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy").unwrap();
        assert_eq!(sk.network(), Testnet);
        assert_eq!(sk.is_compressed(), true);
        assert_eq!(&sk.to_string(), "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy");

        let secp = Secp256k1::new();
        let pk = sk.to_legacy_address(&secp).unwrap();
        assert_eq!(&pk.to_string(), "mqwpxxvfv3QbM8PU8uBx2jaNt9btQqvQNx");

        // mainnet uncompressed
        let sk = Privkey::from_str("5JYkZjmN7PVMjJUfJWfRFwtuXTGB439XV6faajeHPAM9Z2PT2R3").unwrap();
        assert_eq!(sk.network(), Bitcoin);
        assert_eq!(sk.is_compressed(), false);
        assert_eq!(&sk.to_string(), "5JYkZjmN7PVMjJUfJWfRFwtuXTGB439XV6faajeHPAM9Z2PT2R3");

        let secp = Secp256k1::new();
        let pk = sk.to_legacy_address(&secp).unwrap();
        assert_eq!(&pk.to_string(), "1GhQvF6dL8xa6wBxLnWmHcQsurx9RxiMc8");
    }
}
