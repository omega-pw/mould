use serde::Serialize;
use tihu::api::Response;
use tihu_native::ErrNo;

fn err_to_resp(err_no: ErrNo) -> Vec<u8> {
    let err_msg = String::new();
    return (err_msg
        + "{\"code\":"
        + &(err_no.code()).to_string()
        + ",\"message\":\""
        + &err_no.message()
        + "\"}")
        .into_bytes();
}

pub fn json_serialize_err(err: serde_json::Error) -> Vec<u8> {
    log::error!("json序列化失败: {:?}", err);
    return err_to_resp(ErrNo::SerializeError(err));
}

pub fn result_to_json_resp<T: Serialize>(result: Result<T, ErrNo>) -> Vec<u8> {
    let resp: Response<T> = match result {
        Ok(data) => Response::success(Some(data)),
        Err(err_no) => err_no.into(),
    };
    return serde_json::to_vec(&resp).unwrap_or_else(json_serialize_err);
}

pub fn gen_no_such_api() -> Vec<u8> {
    return err_to_resp(ErrNo::NoSuchApi);
}

pub fn gen_login_required() -> Vec<u8> {
    return err_to_resp(ErrNo::LoginRequired);
}

pub fn gen_too_frequent() -> Vec<u8> {
    return err_to_resp(ErrNo::TooFrequent);
}
