use super::hmac_sha;
use byteorder::{BigEndian, ByteOrder};
use hmac_sha::{hmac_sha512, Hash512};

pub fn pbkdf_hmac_sha512(
    password: &[u8],
    salt: &[u8],
    iter_count: u32,
    key_length: u32,
    mk: &mut Vec<u8>,
) {
    //println!("Running key derivation function");

    // TODO: unnecessary cast to u64
    if key_length as u64 > (std::u32::MAX - 1) as u64 * Hash512::DigestSize as u64 {
        panic!("Key length too large");
    }

    // we round up using integer divison - this works for +ive integers and doesn't take
    // into account overflow etc
    let len = (key_length + (Hash512::DigestSize as u32 - 1)) / Hash512::DigestSize as u32;
    let r = (key_length - (len - 1) * Hash512::DigestSize as u32) as usize;
    //println!("r: {}, len: {}, key_length: {}", r, len, key_length);

    // I think the salt needs to be the same length as the output of SHA512
    //assert_eq!(salt.len(), Hash512::DigestSize as usize / 8);

    let mut master_key = vec![0u8; mk.len()];

    #[allow(non_snake_case)]
    for i in 1..=len {
        // T needs to be 512 bits wide
        let mut T = vec![0u64; 8];
        let mut i_bytes = [0u8; 4];
        BigEndian::write_u32(&mut i_bytes, i);
        //println!("i_bytes: {:2x?}", i_bytes);
        //println!("salt: {:2x?}", salt);

        // U = salt || Int(i)
        let mut U = [salt, &i_bytes].concat();
        //println!("U is {:?}", U);
        //println!("password: {:?}", password);
        //println!("iter_count: {}", iter_count);

        for _ in 1..=iter_count {
            let mac = hmac_sha512(password, &U);
            U = vec![0u8; Hash512::DigestSize as usize / 8];
            BigEndian::write_u64_into(&mac, &mut U);
            //println!("mac is {:?}", mac);
            for (i, hash) in mac.iter().enumerate() {
                T[i] ^= hash;
            }
            //println!("T is {:?}", T);
        }
        // convert T to u8
        let mut t_bytes = vec![0u8; (T.len() * 8) as usize];
        BigEndian::write_u64_into(&T, &mut t_bytes);

        master_key.extend(&t_bytes); // [0..r];
    }
    mk.extend(&master_key[0..r / 8]);
    //println!("mk is {:?}", mk);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf_simple() {
        let password = "password".as_bytes();
        let salt = "salt".as_bytes();
        let mut mk: Vec<u8> = Vec::new(); //vec![0u8; (key_length / 8) as usize];
        let mk_expected = vec![
            0x86, 0x7f, 0x70, 0xcf, 0x1a, 0xde, 0x02, 0xcf, 0xf3, 0x75, 0x25, 0x99, 0xa3, 0xa5,
            0x3d, 0xc4, 0xaf, 0x34, 0xc7, 0xa6, 0x69, 0x81, 0x5a, 0xe5, 0xd5, 0x13, 0x55, 0x4e,
            0x1c, 0x8c, 0xf2, 0x52, 0xc0, 0x2d, 0x47, 0x0a, 0x28, 0x5a, 0x05, 0x01, 0xba, 0xd9,
            0x99, 0xbf, 0xe9, 0x43, 0xc0, 0x8f, 0x05, 0x02, 0x35, 0xd7, 0xd6, 0x8b, 0x1d, 0xa5,
            0x5e, 0x63, 0xf7, 0x3b, 0x60, 0xa5, 0x7f, 0xce,
        ];

        // generate the master key
        pbkdf_hmac_sha512(&password, &salt, 1, 512, &mut mk);
        for (i, &byte) in mk_expected.iter().enumerate() {
            assert_eq!(byte, mk[i]);
        }
    }

    #[test]
    fn test_pbkdf_medium() {
        let password = "passDATAb00AB7YxDTT".as_bytes();
        let salt = "saltKEYbcTcXHCBxtjD".as_bytes();
        let mut mk: Vec<u8> = Vec::new();
        let mk_expected = vec![
            0xAC, 0xCD, 0xCD, 0x87, 0x98, 0xAE, 0x5C, 0xD8, 0x58, 0x04, 0x73, 0x90, 0x15, 0xEF,
            0x2A, 0x11, 0xE3, 0x25, 0x91, 0xB7, 0xB7, 0xD1, 0x6F, 0x76, 0x81, 0x9B, 0x30, 0xB0,
            0xD4, 0x9D, 0x80, 0xE1, 0xAB, 0xEA, 0x6C, 0x98, 0x22, 0xB8, 0x0A, 0x1F, 0xDF, 0xE4,
            0x21, 0xE2, 0x6F, 0x56, 0x03, 0xEC, 0xA8, 0xA4, 0x7A, 0x64, 0xC9, 0xA0, 0x04, 0xFB,
            0x5A, 0xF8, 0x22, 0x9F, 0x76, 0x2F, 0xF4, 0x1F,
        ];

        // generate the master key
        pbkdf_hmac_sha512(&password, &salt, 100_000, 512, &mut mk);
        for (i, &byte) in mk.iter().enumerate() {
            assert_eq!(byte, mk_expected[i]);
        }
    }

    #[test]
    #[ignore]
    fn test_pbkdf_complex() {
        let password = "password".as_bytes();
        let salt = "salt".as_bytes();
        let mut mk: Vec<u8> = Vec::new(); //vec![0u8; (key_length / 8) as usize];
        let mk_expected = vec![
            0xAC, 0xCD, 0xCD, 0x87, 0x98, 0xAE, 0x5C, 0xD8, 0x58, 0x04, 0x73, 0x90, 0x15, 0xEF,
            0x2A, 0x11, 0xE3, 0x25, 0x91, 0xB7, 0xB7, 0xD1, 0x6F, 0x76, 0x81, 0x9B, 0x30, 0xB0,
            0xD4, 0x9D, 0x80, 0xE1, 0xAB, 0xEA, 0x6C, 0x98, 0x22, 0xB8, 0x0A, 0x1F, 0xDF, 0xE4,
            0x21, 0xE2, 0x6F, 0x56, 0x03, 0xEC, 0xA8, 0xA4, 0x7A, 0x64, 0xC9, 0xA0, 0x04, 0xFB,
            0x5A, 0xF8, 0x22, 0x9F, 0x76, 0x2F, 0xF4, 0x1F,
        ];

        // generate the master key
        pbkdf_hmac_sha512(&password, &salt, 1, 512, &mut mk);
        for (i, &byte) in mk.iter().enumerate() {
            assert_eq!(byte, mk_expected[i]);
        }
    }
}
