use super::sha_hash;
use byteorder::{BigEndian, ByteOrder};
use sha_hash::sha512;

#[allow(dead_code)]
pub enum Hash512 {
    BlockSize = 1024,
    DigestSize = 512,
}

// See https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.198-1.pdf
// for HMAC construction algorithm and implementation details

// TODO: this currently only works for SHA-512 where the block size is 1024 bits
pub fn hmac_sha512(key: &[u8], text: &[u8]) -> [u64; 8] {
    // determine K0
    use std::cmp::Ordering;

    let mut k0 = [0u8; 128];

    //println!("text is: {:2x?}\n", text);
    //println!("key is: {:2x?}\n", key);
    //println!("------");

    match key.len().cmp(&128) {
        Ordering::Equal => {
            k0.copy_from_slice(key);
        }
        Ordering::Greater => {
            let hashed_key = sha512::hash(key).expect("Couldn't hash key");
            let mut hashed_key_bytes = vec![0u8; hashed_key.len() * 8];
            BigEndian::write_u64_into(&hashed_key, &mut hashed_key_bytes);

            for (i, &byte) in hashed_key_bytes.iter().enumerate() {
                k0[i] = byte;
            }
        }
        Ordering::Less => {
            for (i, &byte) in key.iter().enumerate() {
                k0[i] = byte;
            }
        }
    }
    //println!("k0 is: {:2x?}\n", k0);

    let ipad = vec![0x36; 128];
    let opad = vec![0x5c; 128];

    let mut k0_ipad = vec![0u8; 128];
    let mut k0_opad = vec![0u8; 128];
    assert_eq!(k0.len(), ipad.len());
    assert_eq!(k0.len(), opad.len());

    for (i, item) in k0.iter().enumerate() {
        k0_ipad[i] = ipad[i] ^ item;
        k0_opad[i] = opad[i] ^ item;
    }
    //println!("k0^ipad is: {:2x?}\n", k0_ipad);

    // concat k0 + ipad + text
    k0_ipad.extend(text);
    //println!("(k0^ipad)||text is: {:2x?}\n", k0_ipad);

    // hash and convert to bytes
    let hash = sha512::hash(&k0_ipad).expect("Couldn't hash k0 + ipad + text");
    let mut hash_bytes = vec![0u8; hash.len() * 8];
    BigEndian::write_u64_into(&hash, &mut hash_bytes);
    //println!("hash((k0^ipad)||text) is: {:2x?}\n", hash);

    //println!("k0^opad is: {:2x?}\n", k0_opad);
    // concat (k0 ^ opad) and hash bytes
    k0_opad.extend(&hash_bytes);
    let mac = sha512::hash(&k0_opad).expect("Couldn't do final hash for mac");
    mac
}

#[cfg(test)]
mod tests {
    use super::*;

    // keylen = blocklen
    #[test]
    fn test_hmac_sha512_equal() {
        let message = "Sample message for keylen=blocklen";

        let key: &[u32] = &[
            0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F, 0x10111213, 0x14151617, 0x18191A1B,
            0x1C1D1E1F, 0x20212223, 0x24252627, 0x28292A2B, 0x2C2D2E2F, 0x30313233, 0x34353637,
            0x38393A3B, 0x3C3D3E3F, 0x40414243, 0x44454647, 0x48494A4B, 0x4C4D4E4F, 0x50515253,
            0x54555657, 0x58595A5B, 0x5C5D5E5F, 0x60616263, 0x64656667, 0x68696A6B, 0x6C6D6E6F,
            0x70717273, 0x74757677, 0x78797A7B, 0x7C7D7E7F,
        ];

        let mac_expected: &[u64] = &[
            0xFC25E240658CA785,
            0xB7A811A8D3F7B4CA,
            0x48CFA26A8A366BF2,
            0xCD1F836B05FCB024,
            0xBD36853081811D6C,
            0xEA4216EBAD79DA1C,
            0xFCB95EA4586B8A0C,
            0xE356596A55FB1347,
        ];

        let mut key_bytes = vec![0u8; key.len() * 4];
        BigEndian::write_u32_into(&key, &mut key_bytes);

        let hmac = hmac_sha512(&key_bytes, &message.as_bytes());

        for (i, &hash) in hmac.iter().enumerate() {
            assert_eq!(hash, mac_expected[i]);
        }
    }

    // keylen < blocklen
    #[test]
    fn test_hmac_sha512_less() {
        let message = "Sample message for keylen<blocklen";

        let key: &[u32] = &[
            0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F, 0x10111213, 0x14151617, 0x18191A1B,
            0x1C1D1E1F, 0x20212223, 0x24252627, 0x28292A2B, 0x2C2D2E2F, 0x30313233, 0x34353637,
            0x38393A3B, 0x3C3D3E3F,
        ];

        let mac_expected: &[u64] = &[
            0xFD44C18BDA0BB0A6,
            0xCE0E82B031BF2818,
            0xF6539BD56EC00BDC,
            0x10A8A2D730B3634D,
            0xE2545D639B0F2CF7,
            0x10D0692C72A1896F,
            0x1F211C2B922D1A96,
            0xC392E07E7EA9FEDC,
        ];

        let mut key_bytes = vec![0u8; key.len() * 4];
        BigEndian::write_u32_into(&key, &mut key_bytes);

        let hmac = hmac_sha512(&key_bytes, &message.as_bytes());

        for (i, &hash) in hmac.iter().enumerate() {
            assert_eq!(hash, mac_expected[i]);
        }
    }

    // keylen > blocklen
    #[test]
    fn test_hmac_sha512_greater() {
        let message = "Sample message for keylen=blocklen";

        let key: &[u32] = &[
            0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F, 0x10111213, 0x14151617, 0x18191A1B,
            0x1C1D1E1F, 0x20212223, 0x24252627, 0x28292A2B, 0x2C2D2E2F, 0x30313233, 0x34353637,
            0x38393A3B, 0x3C3D3E3F, 0x40414243, 0x44454647, 0x48494A4B, 0x4C4D4E4F, 0x50515253,
            0x54555657, 0x58595A5B, 0x5C5D5E5F, 0x60616263, 0x64656667, 0x68696A6B, 0x6C6D6E6F,
            0x70717273, 0x74757677, 0x78797A7B, 0x7C7D7E7F, 0x80818283, 0x84858687, 0x88898A8B,
            0x8C8D8E8F, 0x90919293, 0x94959697, 0x98999A9B, 0x9C9D9E9F, 0xA0A1A2A3, 0xA4A5A6A7,
            0xA8A9AAAB, 0xACADAEAF, 0xB0B1B2B3, 0xB4B5B6B7, 0xB8B9BABB, 0xBCBDBEBF, 0xC0C1C2C3,
            0xC4C5C6C7,
        ];

        let mac_expected: &[u64] = &[
            0xD93EC8D2DE1AD2A9,
            0x957CB9B83F14E76A,
            0xD6B5E0CCE285079A,
            0x127D3B14BCCB7AA7,
            0x286D4AC0D4CE6421,
            0x5F2BC9E6870B33D9,
            0x7438BE4AAA20CDA5,
            0xC5A912B48B8E27F3,
        ];

        let mut key_bytes = vec![0u8; key.len() * 4];
        BigEndian::write_u32_into(&key, &mut key_bytes);

        let hmac = hmac_sha512(&key_bytes, &message.as_bytes());

        for (i, &hash) in hmac.iter().enumerate() {
            assert_eq!(hash, mac_expected[i]);
        }
    }

    // keylen < blocklen, with truncated tag
    #[test]
    #[ignore] // not implemented yet
    fn test_hmac_sha512_truncated() {
        let message = "Sample message for keylen<blocklen, with truncated tag";

        let key: &[u32] = &[
            0x00, 0x01020304, 0x05060708, 0x090A0B0C, 0x0D0E0F10, 0x11121314, 0x15161718,
            0x191A1B1C, 0x1D1E1F20, 0x21222324, 0x25262728, 0x292A2B2C, 0x2D2E2F30,
        ];

        let mac_expected: &[u64] = &[
            0x00F3E9A77BB0F06D,
            0xE15F160603E42B50,
            0x28758808596664C0,
            0x3E1AB8FB2B076778,
        ];

        let mut key_bytes = vec![0u8; key.len() * 4];
        BigEndian::write_u32_into(&key, &mut key_bytes);

        let hmac = hmac_sha512(&key_bytes, &message.as_bytes());

        for (i, &hash) in hmac.iter().enumerate() {
            assert_eq!(hash, mac_expected[i]);
        }
    }
}
