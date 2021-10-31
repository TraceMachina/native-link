// Copyright 2020 Nathan (Blaise) Bruer.  All rights reserved.

use std::convert::TryFrom;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};

use hex::FromHex;
use lazy_init::LazyTransform;
pub use log;
use proto::build::bazel::remote::execution::v2::Digest;

use error::{make_input_err, Error, ResultExt};

pub struct DigestInfo {
    // Possibly the size of the digest in bytes. This should only be trusted
    // if `truest_size` is true.
    pub size_bytes: i64,

    // Raw hash in packed form.
    pub packed_hash: [u8; 32],

    // If you can trust the size_bytes to be the size of the data.
    // CAS requests/updates should be true, AC should be false.
    pub trust_size: bool,

    // Cached string representation of the `packed_hash`.
    str_hash: LazyTransform<Option<String>, String>,
}

impl DigestInfo {
    pub fn try_new<T>(hash: &str, size_bytes: T) -> Result<Self, Error>
    where
        T: TryInto<i64> + std::fmt::Display + Copy,
    {
        let packed_hash = <[u8; 32]>::from_hex(hash).err_tip(|| format!("Invalid sha256 hash: {}", hash))?;
        let size_bytes = size_bytes
            .try_into()
            .or_else(|_| Err(make_input_err!("Could not convert {} into i64", size_bytes)))?;
        Ok(DigestInfo {
            size_bytes: size_bytes,
            packed_hash: packed_hash,
            trust_size: false,
            str_hash: LazyTransform::new(None),
        })
    }

    pub fn str<'a>(&'a self) -> &'a str {
        &self
            .str_hash
            .get_or_create(|v| v.unwrap_or_else(|| hex::encode(self.packed_hash)))
    }
}

impl PartialEq for DigestInfo {
    fn eq(&self, other: &Self) -> bool {
        self.size_bytes == other.size_bytes && self.packed_hash == other.packed_hash
    }
}

impl Eq for DigestInfo {}

impl Hash for DigestInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size_bytes.hash(state);
        self.packed_hash.hash(state);
    }
}

impl Clone for DigestInfo {
    fn clone(&self) -> Self {
        DigestInfo {
            size_bytes: self.size_bytes,
            packed_hash: self.packed_hash,
            trust_size: self.trust_size,
            str_hash: LazyTransform::new(None),
        }
    }
}

impl TryFrom<Digest> for DigestInfo {
    type Error = Error;
    fn try_from(digest: Digest) -> Result<Self, Self::Error> {
        let packed_hash =
            <[u8; 32]>::from_hex(&digest.hash).err_tip(|| format!("Invalid sha256 hash: {}", digest.hash))?;
        Ok(DigestInfo {
            size_bytes: digest.size_bytes,
            packed_hash: packed_hash,
            trust_size: false,
            str_hash: LazyTransform::new(Some(digest.hash)),
        })
    }
}

impl Into<Digest> for DigestInfo {
    fn into(self) -> Digest {
        let packed_hash = self.packed_hash;
        let hash = self
            .str_hash
            .into_inner()
            .unwrap_or_else(|v| v.unwrap_or_else(|| hex::encode(packed_hash)));
        Digest {
            hash: hash,
            size_bytes: self.size_bytes,
        }
    }
}
