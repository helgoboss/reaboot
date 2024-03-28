use multihash_codetable::MultihashDigest;
use thiserror::Error;

pub fn verify_source_hash<'a>(
    provided_hash: &'a str,
    file_content: &[u8],
) -> Result<(), VerifySourceHashError<'a>> {
    let provided_hash_bytes =
        hex::decode(provided_hash).map_err(|source| VerifySourceHashError::HashNotHexEncoded {
            provided_hash,
            source,
        })?;
    let provided_multihash = multihash_codetable::Multihash::from_bytes(&provided_hash_bytes)
        .map_err(|source| VerifySourceHashError::InvalidMultiHash {
            provided_hash,
            source,
        })?;
    let provided_multihash_code = provided_multihash.code();
    if provided_multihash_code != SHA256_CODE {
        return Err(VerifySourceHashError::UnsupportedHashAlgorithm {
            code: provided_multihash_code,
        });
    }
    let actual_multihash = multihash_codetable::Code::Sha2_256.digest(file_content);
    if actual_multihash.digest() != provided_multihash.digest() {
        let actual_hash = hex::encode(actual_multihash.to_bytes());
        return Err(VerifySourceHashError::HashesNotMatching {
            provided_hash,
            actual_hash,
        });
    }
    Ok(())
}

pub fn build_sha256_source_hash(bytes: impl AsRef<[u8]>) -> String {
    let multihash = multihash_codetable::Code::Sha2_256.digest(bytes.as_ref());
    hex::encode(multihash.to_bytes())
}

#[derive(Error, Debug)]
pub enum VerifySourceHashError<'a> {
    #[error("provided hash is not properly hex-encoded ('{provided_hash}')")]
    HashNotHexEncoded {
        provided_hash: &'a str,
        source: hex::FromHexError,
    },
    #[error("provided hash is not a valid multi hash ('{provided_hash}')")]
    InvalidMultiHash {
        provided_hash: &'a str,
        source: multihash::Error,
    },
    #[error("provided hash has unsupported multihash algorithm '{code}'")]
    UnsupportedHashAlgorithm { code: u64 },
    #[error("hashes not matching (provided = '{provided_hash}`, actual = '{actual_hash}')")]
    HashesNotMatching {
        provided_hash: &'a str,
        actual_hash: String,
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
    fn verify() {
        let success = verify_source_hash(
            "1220b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "hello world".as_bytes(),
        );
        success.unwrap();
        let wrong_code = verify_source_hash(
            "0220b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "hello world".as_bytes(),
        );
        assert!(matches!(
            wrong_code.unwrap_err(),
            VerifySourceHashError::UnsupportedHashAlgorithm { code: 02, .. }
        ));
    }
}
