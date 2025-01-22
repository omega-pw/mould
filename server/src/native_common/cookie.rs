use log;
use std::collections::HashMap;

//解析请求的cookie
pub fn parse_cookie(cookie_str: &str) -> HashMap<String, String> {
    let mut cookies: HashMap<String, String> = HashMap::new();
    for item in cookie_str.split(";") {
        let key_val: Vec<&str> = item.split("=").collect();
        match (key_val.get(0), key_val.get(1)) {
            (Some(key), Some(val)) => {
                let key = key.trim().to_string();
                let val = val.trim().to_string();
                if !key.is_empty() {
                    cookies.insert(key, val);
                } else {
                    log::warn!("cookie部分格式有误(key为空): {:?}", item);
                }
            }
            _ => {
                log::warn!("cookie部分格式有误: {:?}", item);
            }
        }
    }
    return cookies;
}

#[allow(non_snake_case)]
pub struct CookieAttr {
    pub Expires: Option<String>,
    pub Domain: Option<String>,
    pub Path: Option<String>,
    pub Secure: Option<()>,
    pub HttpOnly: Option<()>,
}

impl CookieAttr {
    pub fn empty() -> CookieAttr {
        return CookieAttr {
            Expires: None,
            Domain: None,
            Path: None,
            Secure: None,
            HttpOnly: None,
        };
    }
}

//格式化响应的cookie
#[allow(non_snake_case)]
pub fn format_cookie(key: &str, val: &str, attr: &CookieAttr) -> String {
    let equal_connector = "=";
    let expires_connector = "; Expires=";
    let domain_connector = "; Domain=";
    let path_connector = "; Path=";
    let secure_connector = "; Secure";
    let http_only_connector = "; HttpOnly";
    let mut capacity = key.len() + equal_connector.len() + val.len();

    match attr.Expires {
        Some(ref Expires) => {
            capacity += expires_connector.len() + Expires.len();
        }
        None => (),
    }
    match attr.Domain {
        Some(ref Domain) => {
            capacity += domain_connector.len() + Domain.len();
        }
        None => (),
    }
    match attr.Path {
        Some(ref Path) => {
            capacity += path_connector.len() + Path.len();
        }
        None => (),
    }
    match attr.Secure {
        Some(ref _Secure) => {
            capacity += secure_connector.len();
        }
        None => (),
    }
    match attr.HttpOnly {
        Some(ref _HttpOnly) => {
            capacity += http_only_connector.len();
        }
        None => (),
    }
    let mut output = String::with_capacity(capacity);
    output.push_str(key);
    output.push_str(equal_connector);
    output.push_str(val);
    match attr.Expires {
        Some(ref Expires) => {
            output.push_str(expires_connector);
            output.push_str(Expires);
        }
        None => (),
    }
    match attr.Domain {
        Some(ref Domain) => {
            output.push_str(domain_connector);
            output.push_str(Domain);
        }
        None => (),
    }
    match attr.Path {
        Some(ref Path) => {
            output.push_str(path_connector);
            output.push_str(Path);
        }
        None => (),
    }
    match attr.Secure {
        Some(ref _Secure) => {
            output.push_str(secure_connector);
        }
        None => (),
    }
    match attr.HttpOnly {
        Some(ref _HttpOnly) => {
            output.push_str(http_only_connector);
        }
        None => (),
    }
    //添加调试断言，让初始化容量绝对准确，做到字符串不扩容
    debug_assert_eq!(capacity, output.len());
    return output;
}
