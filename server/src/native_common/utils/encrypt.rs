use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use crypto::aes::ecb_decryptor;
use crypto::aes::ecb_encryptor;
use crypto::aes::KeySize;
use crypto::blockmodes::PkcsPadding;
use crypto::buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::sha2::Sha256;
use crypto::sha2::Sha512;
use log;
use rand::random;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::Pkcs1v15Encrypt;
use rsa::pkcs1v15::Pkcs1v15Sign;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePublicKey;
use rsa::{RsaPrivateKey, RsaPublicKey};
use signature::RandomizedSigner;
use signature::SignatureEncoding;
use std::fmt;
use std::fs;
use std::path::Path;
use tihu::LightString;

pub fn fill_random_bytes(data: &mut [u8]) {
    for ch in data.iter_mut() {
        *ch = random();
    }
}

pub fn encrypt_by_base64(plain: &[u8]) -> Result<String, LightString> {
    return Ok(BASE64_STANDARD.encode(plain));
}

pub fn decrypt_by_base64(cipher: &str) -> Result<Vec<u8>, LightString> {
    return BASE64_STANDARD
        .decode(cipher)
        .map_err(|err| -> LightString {
            log::error!("base64 decode failed: {}, original data: {}", err, cipher);
            LightString::from_static("base64 decode failed")
        });
}

pub struct HexStr<'a>(pub &'a [u8]);

impl fmt::Display for HexStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.input(data);
    let mut out: [u8; 20] = [0; 20];
    hasher.result(&mut out);
    return out;
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.input(data);
    let mut out: [u8; 32] = [0; 32];
    hasher.result(&mut out);
    return out;
}

pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.input(data);
    let mut out: [u8; 64] = [0; 64];
    hasher.result(&mut out);
    return out;
}

pub fn new_rsa_key_pair() -> Result<(RsaPublicKey, RsaPrivateKey), LightString> {
    let mut rng = rand::thread_rng();
    let rsa_pri_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|err| -> LightString {
        log::error!("generate rsa key pair failed: {}", err);
        LightString::from_static("generate rsa key pair failed")
    })?;
    let rsa_pub_key = rsa_pri_key.to_public_key();
    return Ok((rsa_pub_key, rsa_pri_key));
}

pub fn read_rsa_pub_key<P: AsRef<Path>>(rsa_pub_key: P) -> Result<RsaPublicKey, LightString> {
    let content = fs::read_to_string(rsa_pub_key.as_ref()).map_err(|err| -> LightString {
        log::error!("read public key error: {}", err);
        LightString::from_static("read public key error")
    })?;
    return RsaPublicKey::from_public_key_pem(&content).map_err(|err| -> LightString {
        log::error!(
            "public key is invalid: {}, key path: {:?}",
            err,
            rsa_pub_key.as_ref()
        );
        LightString::from_static("public key is invalid")
    });
}

pub fn new_rsa_pub_key(content: &str) -> Result<RsaPublicKey, LightString> {
    return RsaPublicKey::from_public_key_pem(content).map_err(|err| -> LightString {
        log::error!("public key is invalid: {}, content: {}", err, content);
        LightString::from_static("public key is invalid")
    });
}

pub fn read_rsa_pri_key<P: AsRef<Path>>(rsa_pri_key: P) -> Result<RsaPrivateKey, LightString> {
    let content = fs::read_to_string(rsa_pri_key.as_ref()).map_err(|err| -> LightString {
        log::error!("read private key error: {}", err);
        LightString::from_static("read private key error")
    })?;
    return RsaPrivateKey::from_pkcs1_pem(&content).map_err(|err| -> LightString {
        log::error!(
            "private key is invalid: {}, key path: {:?}",
            err,
            rsa_pri_key.as_ref()
        );
        LightString::from_static("private key is invalid")
    });
}

pub fn new_rsa_pri_key(content: &str) -> Result<RsaPrivateKey, LightString> {
    return RsaPrivateKey::from_pkcs1_pem(content).map_err(|err| -> LightString {
        log::error!("private key is invalid: {}, content: {}", err, content);
        LightString::from_static("private key is invalid")
    });
}

pub fn encrypt_by_rsa_pub_key(
    data: &[u8],
    rsa_pub_key: &RsaPublicKey,
) -> Result<Vec<u8>, LightString> {
    let mut rng = rand::thread_rng();
    return rsa_pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .map_err(|err| -> LightString {
            log::error!("public encrypt failed: {}", err);
            LightString::from_static("public encrypt failed")
        });
}

// pub fn encrypt_by_rsa_pri_key(data: &str, rsa_pri_key: &Rsa<Private>) -> Result<String, LightString> {
//     let mut output = vec![0; rsa_pri_key.size() as usize];
//     rsa_pri_key
//         .private_encrypt(data.as_bytes(), &mut output, Padding::PKCS1)
//         .map_err(|err| -> LightString {
//             log::error!("private encrypt failed: {}", err);
//             "private encrypt failed".into()
//         })?;
//     return Ok(base62::encode(&output));
// }

// pub fn decrypt_by_rsa_pub_key(cipher: &str, rsa_pub_key: &Rsa<Public>) -> Result<String, LightString> {
//     //先对密文进行base62解码
//     let cipher_raw = base62::decode(cipher).map_err(|err| -> LightString {
//         log::error!("base62 decode failed: {}, original data: {}", err, cipher);
//         "base62 decode failed".into()
//     })?;
//     let mut output = vec![0; rsa_pub_key.size() as usize];
//     let ret_len = rsa_pub_key
//         .public_decrypt(&cipher_raw, &mut output, Padding::PKCS1)
//         .map_err(|err| -> LightString {
//             log::error!("public decrypt failed: {}", err);
//             "public decrypt failed".into()
//         })?;
//     output.truncate(ret_len as usize);
//     return Ok(String::from_utf8_lossy(&output).into_owned());
// }

pub fn decrypt_by_rsa_pri_key(
    cipher: &[u8],
    rsa_pri_key: &RsaPrivateKey,
) -> Result<Vec<u8>, LightString> {
    return rsa_pri_key
        .decrypt(Pkcs1v15Encrypt, cipher)
        .map_err(|err| -> LightString {
            log::error!("private decrypt failed: {}", err);
            LightString::from_static("private decrypt failed")
        });
}

//用私钥对数据进行签名
pub fn sign_by_rsa_pri_key(
    data: &[u8],
    pri_key: RsaPrivateKey,
) -> Result<Vec<u8>, rsa::errors::Error> {
    let signing_key = SigningKey::<sha2::Sha256>::new(pri_key);
    let mut rng = rand::thread_rng();
    let signature = signing_key.sign_with_rng(&mut rng, data);
    return Ok(signature.to_vec());
}

//用公钥对私钥签名进行验证
pub fn verify_by_pub_pri_key(
    data: &[u8],
    signature: &[u8],
    pub_key: &RsaPublicKey,
) -> Result<(), rsa::errors::Error> {
    let hash = sha256(data);
    return pub_key.verify(Pkcs1v15Sign::new::<sha2::Sha256>(), &hash, signature);
}

pub fn encrypt_by_aes_256(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, LightString> {
    let mut encryptor = ecb_encryptor(KeySize::KeySize256, key, PkcsPadding);
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);
    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|err| -> LightString {
                log::error!("Aes encrypt failed: {:?}", err);
                LightString::from_static("Aes encrypt failed")
            })?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    Ok(final_result)
}

pub fn decrypt_by_aes_256(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, LightString> {
    let mut decryptor = ecb_decryptor(KeySize::KeySize256, key, PkcsPadding);
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);
    loop {
        let result = decryptor
            .decrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|err| -> LightString {
                log::error!("Aes decrypt failed: {:?}", err);
                LightString::from_static("Aes decrypt failed")
            })?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    Ok(final_result)
}
