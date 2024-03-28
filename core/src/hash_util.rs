use multihash::Multihash;
use multihash_codetable::MultihashDigest;
use sha2::{Digest, Sha256, Sha512};
use thiserror::Error;

pub struct ReabootHashVerifier<'a> {
    expected_hash: &'a str,
    expected_multihash: Multihash<64>,
    hasher: ReabootHasher,
}

impl<'a> ReabootHashVerifier<'a> {
    pub fn try_from_hash(expected_hash: &'a str) -> Result<Self, BuildHashVerifierError> {
        let expected_multihash =
            parse_hash(&expected_hash).map_err(BuildHashVerifierError::Parse)?;
        let hasher = ReabootHasher::try_from_multihash(&expected_multihash)?;
        let verifier = Self {
            expected_hash,
            expected_multihash,
            hasher,
        };
        Ok(verifier)
    }

    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub fn verify(self) -> Result<(), HashVerificationError> {
        let actual_digest = self.hasher.finalize();
        if actual_digest != self.expected_multihash.digest() {
            let actual_hash = hex::encode(actual_digest);
            return Err(HashVerificationError::HashesNotMatching {
                expected_hash: self.expected_hash.to_string(),
                actual_hash,
            });
        }
        Ok(())
    }
}

struct ReabootHasher {
    inner: sha2::Sha256,
}

impl ReabootHasher {
    pub fn try_from_multihash(multihash: &Multihash<64>) -> Result<Self, BuildHashVerifierError> {
        let multihash_code = multihash.code();
        if multihash_code != SHA256_CODE {
            return Err(BuildHashVerifierError::UnsupportedHashAlgorithm {
                code: multihash_code,
            });
        }
        let hasher = Self {
            inner: sha2::Sha256::new(),
        };
        Ok(hasher)
    }

    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    pub fn finalize(self) -> Vec<u8> {
        self.inner.finalize().to_vec()
    }
}

pub fn parse_hash(provided_hash: &str) -> Result<Multihash<64>, ParseHashError> {
    let provided_hash_bytes =
        hex::decode(provided_hash).map_err(|source| ParseHashError::HashNotHexEncoded {
            provided_hash: provided_hash.to_string(),
            source,
        })?;
    let provided_multihash = multihash_codetable::Multihash::from_bytes(&provided_hash_bytes)
        .map_err(|source| ParseHashError::InvalidMultiHash {
            provided_hash: provided_hash.to_string(),
            source,
        })?;
    Ok(provided_multihash)
}

pub fn build_sha256_source_hash(bytes: impl AsRef<[u8]>) -> String {
    let multihash = multihash_codetable::Code::Sha2_256.digest(bytes.as_ref());
    hex::encode(multihash.to_bytes())
}

#[derive(Error, Debug)]
pub enum HashVerificationError {
    #[error("hashes not matching (expected = '{expected_hash}`, actual = '{actual_hash}')")]
    HashesNotMatching {
        expected_hash: String,
        actual_hash: String,
    },
}

#[derive(Error, Debug)]
pub enum BuildHashVerifierError {
    #[error("parsing hash failed")]
    Parse(ParseHashError),
    #[error("provided hash has unsupported multihash algorithm '{code}'")]
    UnsupportedHashAlgorithm { code: u64 },
}

#[derive(Error, Debug)]
pub enum ParseHashError {
    #[error("provided hash is not properly hex-encoded ('{provided_hash}')")]
    HashNotHexEncoded {
        provided_hash: String,
        source: hex::FromHexError,
    },
    #[error("provided hash is not a valid multi hash ('{provided_hash}')")]
    InvalidMultiHash {
        provided_hash: String,
        source: multihash::Error,
    },
}

const SHA256_CODE: u64 = 0x12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(
            build_sha256_source_hash(""),
            "1220e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn digest_twice() {
        assert_eq!(build_sha256_source_hash(""), build_sha256_source_hash(""));
    }

    #[test]
    fn single_chunk() {
        assert_eq!(
            build_sha256_source_hash("hello world"),
            "1220b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn verify_ok() {
        let mut verifier = ReabootHashVerifier::try_from_hash(
            "1220b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        )
            .unwrap();
        verifier.update("hello world".as_bytes());
        verifier.verify().unwrap();
    }

    #[test]
    fn verify_err() {
        let mut verifier = ReabootHashVerifier::try_from_hash(
            "1220b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        )
            .unwrap();
        verifier.update("hello world oh no".as_bytes());
        verifier.verify().unwrap_err();
    }
}
