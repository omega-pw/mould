pub mod change_password;
pub mod get_curr_user;
pub mod get_nonce;
pub mod get_openid_providers;
pub mod get_rsa_pub_key;
pub mod get_salt;
pub mod login;
pub mod login_by_oauth2_code;
pub mod login_by_openid_code;
pub mod logout;
pub mod register;
pub mod reset_password;
pub mod send_email_captcha;
use super::utils::pbkdf2;
use tihu::LightString;

pub enum RandomValue<'a, const N: usize> {
    Client([u8; N]),
    Server(&'a [u8], [u8; N]),
}

fn concat_by_copy(trunks: &[&[u8]]) -> Vec<u8> {
    let total_len = trunks
        .iter()
        .map(|item| item.len())
        .fold(0, |prev_len, len| prev_len + len);
    if 0 == total_len {
        return Vec::new();
    }
    //vec!宏针对u8、i8类型的数据有专门的优化
    let mut result = vec![0; total_len];
    let mut pos = 0;
    for trunk in trunks {
        let len = trunk.len();
        let dest = &mut result[pos..pos + len];
        dest.copy_from_slice(trunk);
        pos += len;
    }
    return result;
}

/**
 * 计算盐值
 */
pub fn calc_salt(
    random_value: RandomValue<32>,
    sha512: impl Fn(&[u8]) -> [u8; 64],
) -> Result<[u8; 64], LightString> {
    let data = match random_value {
        RandomValue::Client(client_random_value) => concat_by_copy(&[&client_random_value]),
        RandomValue::Server(account, server_random_value) => {
            concat_by_copy(&[&server_random_value, account])
        }
    };
    return Ok(sha512(&data));
}

pub fn calc_derived_key(password: &[u8], salt: &[u8]) -> ([u8; 32], [u8; 32]) {
    let capacity: u32 = 64;
    let mut derived_key: Vec<u8> = Vec::with_capacity(capacity as usize);
    pbkdf2::pbkdf_hmac_sha512(password, salt, 100000, capacity * 8, &mut derived_key);
    let mut first_part = [0u8; 32];
    let mut second_part = [0u8; 32];
    first_part.copy_from_slice(&derived_key[0..32]);
    second_part.copy_from_slice(&derived_key[32..64]);
    return (first_part, second_part);
}
