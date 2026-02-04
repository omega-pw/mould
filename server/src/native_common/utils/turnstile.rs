use serde::{Deserialize, Serialize};
use std::time::Duration;
use tihu_native::ErrNo;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValidateReq {
    pub secret: String,
    pub response: String,
    pub remoteip: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValidateResp {
    pub success: bool,
}

pub async fn validate_token(
    secret: String,
    token: String,
    remoteip: Option<String>,
    rpc_timeout: u64,
) -> Result<bool, ErrNo> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(rpc_timeout))
        .build()
        .map_err(|err| ErrNo::Other(err.into()))?;
    let resp = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .json(&ValidateReq {
            secret: secret,
            response: token.clone(),
            remoteip: remoteip.clone(),
        })
        .send()
        .await
        .map_err(|err| ErrNo::ApiError(err.into()))?;
    let resp_body: String = resp
        .text()
        .await
        .map_err(|err| ErrNo::ApiError(err.into()))?;
    let resp: ValidateResp = serde_json::from_str(&resp_body).map_err(ErrNo::DeserializeError)?;
    if !resp.success {
        log::warn!(
            "turnstile校验不通过, token: {}, remoteip: {:?}，响应：{}",
            token,
            remoteip,
            resp_body
        );
    }
    return Ok(resp.success);
}
