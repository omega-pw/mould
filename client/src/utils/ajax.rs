use super::gen_id;
use crate::LightString;
use js_sys::decode_uri_component;
use js_sys::Function;
use js_sys::Promise;
use js_sys::Uint8Array;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Event;
use web_sys::File;
use web_sys::FormData;
use web_sys::ProgressEvent;
use web_sys::XmlHttpRequest;
use web_sys::XmlHttpRequestResponseType;

#[derive(Default)]
pub struct RequestOption {
    pub method: Option<String>,
    pub with_credentials: Option<bool>,
    pub headers: Option<HashMap<String, Vec<String>>>,
    pub on_progress: Option<Box<dyn FnMut(f64, f64)>>,
    pub on_upload_progress: Option<Box<dyn FnMut(f64, f64)>>,
}

pub enum Request<Req: Serialize> {
    Json(Req),
    Form(FormData),
}

pub trait ResponseExtractor {
    type Output;
    fn response_type() -> XmlHttpRequestResponseType;
    fn extract(xhr: &XmlHttpRequest) -> Result<Self::Output, LightString>
    where
        Self: Sized;
}

impl ResponseExtractor for Uint8Array {
    type Output = Self;
    fn response_type() -> XmlHttpRequestResponseType {
        XmlHttpRequestResponseType::Arraybuffer
    }
    fn extract(xhr: &XmlHttpRequest) -> Result<Self::Output, LightString> {
        let response = xhr.response().map_err(|err| -> LightString {
            log::error!("获取buffer响应失败: {:?}", err);
            return "获取响应失败".into();
        })?;
        let response = Uint8Array::new(&response);
        return Ok(response);
    }
}

pub struct Json<T>(T);

impl<T> ResponseExtractor for Json<T>
where
    T: DeserializeOwned,
{
    type Output = T;
    fn response_type() -> XmlHttpRequestResponseType {
        XmlHttpRequestResponseType::Json
    }
    fn extract(xhr: &XmlHttpRequest) -> Result<T, LightString> {
        let response = xhr.response().map_err(|err| -> LightString {
            log::error!("获取json响应失败: {:?}", err);
            return "获取响应失败".into();
        })?;
        let result = serde_wasm_bindgen::from_value(response);
        return result.map_err(|err| -> LightString {
            log::error!("Deserialize response data failed: {:?}", err);
            return "Deserialize response data failed!".into();
        });
    }
}

fn get_file_name_from_header(content_disposition: Option<&str>) -> Option<&str> {
    if let Some(content_disposition) = content_disposition {
        let name = content_disposition.split("filename=").last();
        if let Some(mut name) = name {
            if name.starts_with("\"") {
                name = name.split_at(1).1;
            }
            if name.ends_with("\"") {
                name = name.split_at(name.len() - 1).0;
            }
            return Some(name.trim());
        }
    }
    return None;
}

fn get_file_name_from_url(url: &str, allow_no_suffix: bool) -> Result<Option<String>, LightString> {
    let url = decode_uri_component(url).map_err(|err| -> LightString {
        log::error!("根据url获取文件名称时url解码失败: {:?}", err);
        return "获取文件名称失败".into();
    })?;
    if let Some(url) = url.as_string() {
        let uri = url.split("?").next();
        let file_name = uri.map(|uri| uri.split("/").last()).flatten();
        if let Some(file_name) = file_name {
            if !allow_no_suffix && file_name.find(".").is_none() {
                return Ok(None);
            } else {
                return Ok(Some(file_name.trim().to_string()));
            }
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    }
}

impl ResponseExtractor for File {
    type Output = Self;
    fn response_type() -> XmlHttpRequestResponseType {
        XmlHttpRequestResponseType::Blob
    }
    fn extract(xhr: &XmlHttpRequest) -> Result<Self::Output, LightString> {
        let url = xhr.response_url();
        let response = xhr.response().map_err(|err| -> LightString {
            log::error!("获取文件响应失败: {:?}", err);
            return "获取响应失败".into();
        })?;
        let content_disposition =
            xhr.get_response_header("content-disposition")
                .map_err(|err| -> LightString {
                    log::error!("获取content-disposition响应头失败: {:?}", err);
                    return "获取文件名称失败".into();
                })?;
        let file_name =
            if let Some(file_name) = get_file_name_from_header(content_disposition.as_deref()) {
                Some(file_name.to_string())
            } else {
                let file_name = get_file_name_from_url(&url, false)?;
                file_name
            };
        let file_name = file_name.unwrap_or_else(gen_id);
        let file =
            File::new_with_blob_sequence(&response, &file_name).map_err(|err| -> LightString {
                log::error!("构造文件对象失败: {:?}", err);
                return "获取响应文件失败".into();
            })?;
        return Ok(file);
    }
}

fn try_ajax<Req: Serialize, Resp: ResponseExtractor>(
    url: &str,
    req: &Option<Request<Req>>,
    option: &mut Option<RequestOption>,
    resolve: Function,
    reject: Function,
) -> Result<(), LightString> {
    let xhr = XmlHttpRequest::new().map_err(|err| -> LightString {
        log::error!("构造请求对象失败: {:?}", err);
        return "请求失败".into();
    })?;
    xhr.set_response_type(Resp::response_type());

    let onload: Function = {
        let xhr = xhr.clone();
        Closure::wrap(Box::new(move || {
            if let Err(err) = resolve.call1(&wasm_bindgen::JsValue::UNDEFINED, &xhr) {
                log::error!("调用Promise的resolve失败: {:?}", err);
            }
        }) as Box<dyn FnMut()>)
        .into_js_value()
        .dyn_into()
        .map_err(|err| -> LightString {
            log::error!("构造load事件回调失败: {:?}", err);
            return "请求失败".into();
        })?
    };
    xhr.set_onload(Some(&onload));

    if let Some(mut on_progress) = option
        .as_mut()
        .map(|option| option.on_progress.take())
        .flatten()
    {
        let onprogress: Function = Closure::wrap(Box::new(move |event: ProgressEvent| {
            on_progress(event.loaded(), event.total());
        }) as Box<dyn FnMut(ProgressEvent)>)
        .into_js_value()
        .dyn_into()
        .map_err(|err| -> LightString {
            log::error!("构造progress事件回调失败: {:?}", err);
            return "请求失败".into();
        })?;
        xhr.set_onprogress(Some(&onprogress));
    }
    if let Some(mut on_upload_progress) = option
        .as_mut()
        .map(|option| option.on_upload_progress.take())
        .flatten()
    {
        let upload = xhr.upload().map_err(|err| -> LightString {
            log::error!("获取上传对象失败: {:?}", err);
            return "请求失败".into();
        })?;
        let on_upload_progress: Function = Closure::wrap(Box::new(move |event: ProgressEvent| {
            on_upload_progress(event.loaded(), event.total());
        })
            as Box<dyn FnMut(ProgressEvent)>)
        .into_js_value()
        .dyn_into()
        .map_err(|err| -> LightString {
            log::error!("构造上传对象的progress事件回调失败: {:?}", err);
            return "请求失败".into();
        })?;
        upload.set_onprogress(Some(&on_upload_progress));
    }

    let onerror: Function = {
        let reject = reject.clone();
        Closure::wrap(Box::new(move |event: Event| {
            if let Err(err) = reject.call1(&wasm_bindgen::JsValue::UNDEFINED, &event) {
                log::error!("调用Promise的reject失败: {:?}", err);
            }
        }) as Box<dyn FnMut(Event)>)
        .into_js_value()
        .dyn_into()
        .map_err(|err| -> LightString {
            log::error!("构造error事件回调失败: {:?}", err);
            return "请求失败".into();
        })?
    };
    xhr.set_onerror(Some(&onerror));

    let onabort: Function = Closure::wrap(Box::new(move |event: Event| {
        if let Err(err) = reject.call1(&wasm_bindgen::JsValue::UNDEFINED, &event) {
            log::error!("调用Promise的reject失败: {:?}", err);
        }
    }) as Box<dyn FnMut(Event)>)
    .into_js_value()
    .dyn_into()
    .map_err(|err| -> LightString {
        log::error!("构造abort事件回调失败: {:?}", err);
        return "请求失败".into();
    })?;
    xhr.set_onabort(Some(&onabort));

    let method = option
        .as_ref()
        .map(|option| option.method.as_deref())
        .flatten()
        .unwrap_or_else(|| "POST");
    // if (req && "GET" == method) {
    //     let queryString = req.serialize().content;
    //     if (queryString) {
    //         if (url.includes("?")) {
    //             url = url + "&" + queryString;
    //         } else {
    //             url = url + "?" + queryString;
    //         }
    //     }
    // }
    xhr.open_with_async(method, url, true)
        .map_err(|err| -> LightString {
            log::error!("发起ajax请求失败: {:?}", err);
            return "发起请求失败".into();
        })?;
    if let Some(with_credentials) = option
        .as_ref()
        .map(|option| option.with_credentials)
        .flatten()
    {
        xhr.set_with_credentials(with_credentials);
    }
    if let Some(headers) = option
        .as_ref()
        .map(|option| option.headers.as_ref())
        .flatten()
    {
        for (key, values) in headers {
            for value in values {
                xhr.set_request_header(&key, &value)
                    .map_err(|err| -> LightString {
                        log::error!("添加请求头{}失败: {:?}", key, err);
                        return "请求失败".into();
                    })?;
            }
        }
    }
    if let Some(req) = req {
        match req {
            Request::Json(data) => {
                let json = serde_json::to_string(&data).map_err(|err| {
                    log::error!("序列化请求数据失败: {:?}", err);
                    LightString::from("Serialize request data failed!")
                })?;
                xhr.set_request_header("Content-Type", "application/json")
                    .map_err(|err| -> LightString {
                        log::error!("设置json请求头失败: {:?}", err);
                        return "请求失败".into();
                    })?;
                xhr.send_with_opt_str(Some(&json))
                    .map_err(|err| -> LightString {
                        log::error!("发送json请求体失败: {:?}", err);
                        return "请求失败".into();
                    })?;
            }
            Request::Form(form_data) => {
                xhr.send_with_opt_form_data(Some(&form_data))
                    .map_err(|err| -> LightString {
                        log::error!("发送表单请求体失败: {:?}", err);
                        return "请求失败".into();
                    })?;
            }
        }
    } else {
        xhr.send().map_err(|err| -> LightString {
            log::error!("发送请求失败: {:?}", err);
            return "请求失败".into();
        })?;
    }
    return Ok(());
}

pub async fn ajax<Req, Resp>(
    url: String,
    req: Option<Request<Req>>,
    mut option: Option<RequestOption>,
) -> Result<Resp::Output, LightString>
where
    Req: Serialize,
    Resp: ResponseExtractor,
{
    let mut promise_fn = |resolve: Function, reject: Function| {
        if let Err(err) = try_ajax::<Req, Resp>(&url, &req, &mut option, resolve, reject.clone()) {
            if let Err(err) =
                reject.call1(&wasm_bindgen::JsValue::UNDEFINED, &JsValue::from_str(&err))
            {
                log::error!("调用Promise的reject失败: {:?}", err);
            }
        }
    };
    let promise = Promise::new(&mut promise_fn);
    let xhr: XmlHttpRequest = JsFuture::from(promise)
        .await
        .map_err(|err| -> LightString {
            log::error!("ajax请求失败: {:?}", err);
            return "请求失败".into();
        })?
        .dyn_into()
        .map_err(|err| -> LightString {
            log::error!("请求完成输出的不是XmlHttpRequest: {:?}", err);
            return "请求失败".into();
        })?;
    let status = xhr.status().map_err(|err| -> LightString {
        log::error!("获取响应码失败: {:?}", err);
        return "请求失败".into();
    })?;
    if 200 <= status && 300 > status {
        return Resp::extract(&xhr);
    } else if 0 == status {
        log::error!("ajax请求异常, 状态码为0");
        return Err(LightString::from("network error"));
    } else {
        log::error!("ajax请求失败: {:?}", xhr);
        return Err(LightString::from("server error"));
    }
}
