use crate::SharedString;
use futures::lock::Mutex;
use gloo::timers::callback::Timeout;
use js_sys::Date;
use js_sys::Function;
use js_sys::Promise;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::{Arc, LazyLock, RwLock};
use tihu::api::CacheMeta;
use tihu::Api;
use wasm_bindgen_futures::JsFuture;

struct CacheData<D> {
    data: D,
    time: f64,
    timeout: Option<u64>,
}

impl<D> CacheData<D> {
    fn expired(&self) -> bool {
        if let Some(timeout) = self.timeout {
            Date::now() > self.time + timeout as f64
        } else {
            false
        }
    }
}

static CACHE_MAP: LazyLock<Mutex<HashMap<String, CacheData<serde_json::Value>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub trait Handler<In, Out>: 'static {
    fn handle(&self, input: In) -> Pin<Box<dyn Future<Output = Out>>>;
}

impl<In, Out, T> Handler<In, Out> for T
where
    T: Fn(In) -> Pin<Box<dyn Future<Output = Out>>> + 'static,
{
    fn handle(&self, input: In) -> Pin<Box<dyn Future<Output = Out>>> {
        self(input)
    }
}

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

pub type HttpRequestor =
    dyn Handler<(SharedString, SharedString), Result<SharedString, SharedString>> + Send + Sync;
//请求校验器
pub type RequestValidator<T> = dyn Fn(&T) -> Result<(), SharedString>;
//loading展示器
pub type LoadingHandler = dyn Handler<bool, ()> + Send + Sync;
//锁处理器，lock参数为true表示加锁，lock参数为false表示解锁，返回true表示执行成功，返回false表示执行失败
pub type LockHandler = dyn Handler<bool, bool>;
//数据解包器，输入请求器返回的结果，输出Result<成功结果，业务错误>
pub type DataUnwrapper =
    dyn Handler<serde_json::Value, Result<serde_json::Value, SharedString>> + Send + Sync;
//错误处理器
pub type ErrorHandler = dyn Handler<SharedString, ()> + Send + Sync;
//加锁错误处理器
pub type LockErrorHandler = dyn Handler<(), ()>;

pub async fn wait(millis: u32) {
    let mut timeout = None;
    let mut promise_fn = |resolve: Function, _reject: Function| {
        timeout.replace(Timeout::new(millis, move || {
            resolve.call0(&wasm_bindgen::JsValue::UNDEFINED).unwrap();
        }));
    };
    let promise = Promise::new(&mut promise_fn);
    JsFuture::from(promise).await.unwrap();
    timeout.take();
}

pub async fn try_max_times<D, E, F, T>(
    task: T,
    max_times: NonZeroU32,
    interval: Option<u32>,
) -> Result<D, E>
where
    F: Future<Output = Result<D, E>>,
    T: Fn(Option<E>) -> F,
{
    let extra_times = max_times.get() as usize - 1;
    match task(None).await {
        Ok(ret) => {
            return Ok(ret);
        }
        Err(mut error) => {
            for _ in 0..extra_times {
                if let Some(interval) = interval {
                    wait(interval).await;
                }
                match task(Some(error)).await {
                    Ok(ret) => {
                        return Ok(ret);
                    }
                    Err(latest_err) => {
                        error = latest_err;
                    }
                }
            }
            return Err(error);
        }
    }
}

pub struct Request<Req, Resp> {
    url: SharedString,
    method: Method,
    http_requestor: Option<Arc<HttpRequestor>>,
    show_loading: bool,
    request_validator: Option<Arc<RequestValidator<Req>>>,
    loading_handler: Option<Arc<LoadingHandler>>,
    lock_handler: Option<Arc<LockHandler>>,
    data_unwrapper: Option<Arc<DataUnwrapper>>,
    validate_error_handler: Option<Arc<ErrorHandler>>,
    lock_error_handler: Option<Arc<LockErrorHandler>>,
    req_error_handler: Option<Arc<ErrorHandler>>,
    unwrap_error_handler: Option<Arc<ErrorHandler>>,
    get_cache_meta: Option<Box<dyn Fn(&Req) -> Option<CacheMeta>>>,
    max_times: Option<NonZeroU32>,
    interval: Option<u32>,
    phantom1: PhantomData<Req>,
    phantom2: PhantomData<Resp>,
}

impl<Req, Resp> Request<Req, Resp>
where
    Req: Serialize + 'static,
    Resp: DeserializeOwned + 'static,
{
    pub fn new(url: SharedString) -> Self {
        Self {
            url: url,
            method: Method::Post,
            http_requestor: None,
            show_loading: true,
            request_validator: None,
            loading_handler: None,
            lock_handler: None,
            data_unwrapper: None,
            validate_error_handler: None,
            lock_error_handler: None,
            req_error_handler: None,
            unwrap_error_handler: None,
            get_cache_meta: None,
            max_times: None,
            interval: None,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
    pub fn http_requestor(
        &mut self,
        http_requestor: impl Handler<(SharedString, SharedString), Result<SharedString, SharedString>>
            + Send
            + Sync,
    ) -> &mut Self {
        self.http_requestor = Some(Arc::new(http_requestor));
        return self;
    }
    pub fn data_unwrapper(
        &mut self,
        data_unwrapper: impl Handler<serde_json::Value, Result<serde_json::Value, SharedString>>
            + Send
            + Sync,
    ) -> &mut Self {
        self.data_unwrapper = Some(Arc::new(data_unwrapper));
        return self;
    }
    pub fn lock_handler(&mut self, lock_handler: impl Handler<bool, bool>) -> &mut Self {
        self.lock_handler = Some(Arc::new(lock_handler));
        return self;
    }
    pub fn request_validator(
        &mut self,
        request_validator: impl Fn(&Req) -> Result<(), SharedString> + 'static,
    ) -> &mut Self {
        self.request_validator = Some(Arc::new(request_validator));
        return self;
    }
    pub fn loading_handler(
        &mut self,
        loading_handler: impl Handler<bool, ()> + Send + Sync,
    ) -> &mut Self {
        self.loading_handler = Some(Arc::new(loading_handler));
        return self;
    }
    pub fn disable_loading(&mut self) -> &mut Self {
        self.show_loading = false;
        return self;
    }
    pub fn lock_error_handler(&mut self, lock_error_handler: impl Handler<(), ()>) -> &mut Self {
        self.lock_error_handler = Some(Arc::new(lock_error_handler));
        return self;
    }
    pub fn req_error_handler(
        &mut self,
        req_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> &mut Self {
        self.req_error_handler = Some(Arc::new(req_error_handler));
        return self;
    }
    pub fn unwrap_error_handler(
        &mut self,
        unwrap_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> &mut Self {
        self.unwrap_error_handler = Some(Arc::new(unwrap_error_handler));
        return self;
    }
    pub fn cache_by_key(
        &mut self,
        key: impl Into<tihu::SharedString>,
        ttl: Option<u64>,
    ) -> &mut Self {
        let key: tihu::SharedString = key.into();
        self.get_cache_meta.replace(Box::new(move |_req: &Req| {
            Some(CacheMeta {
                key: key.clone(),
                ttl: ttl,
            })
        }));
        return self;
    }
    pub fn try_max_times(&mut self, max_times: u32, interval: Option<u32>) -> &mut Self {
        self.max_times = NonZeroU32::new(max_times);
        self.interval = interval;
        return self;
    }
    pub async fn call(&self, req: &Req) -> Result<Resp, SharedString> {
        if let Some(request_validator) = self.request_validator.as_ref() {
            if let Err(err_msg) = request_validator(req) {
                if let Some(validate_error_handler) =
                    self.validate_error_handler.clone().or_else(|| {
                        LazyLock::force(&DEFAULT_VALIDATE_ERROR_HANDLER)
                            .read()
                            .unwrap()
                            .clone()
                    })
                {
                    validate_error_handler.handle(err_msg.clone()).await;
                }
                return Err(err_msg);
            }
        }
        let mut cache_meta = None;
        if let Some(get_cache_meta) = self.get_cache_meta.as_ref() {
            cache_meta = get_cache_meta(req);
        }
        if let Some(cache_meta) = cache_meta.as_ref() {
            let cache_map = LazyLock::force(&CACHE_MAP);
            let cache_map = cache_map.lock().await;
            if let Some(cache) = cache_map.get(cache_meta.key.as_str()) {
                if !cache.expired() {
                    if let Ok(resp) = serde_json::from_value::<Resp>(cache.data.clone()) {
                        return Ok(resp);
                    }
                }
            }
        }
        return self.call_with_lock(req, cache_meta).await;
    }

    async fn call_with_lock(
        &self,
        req: &Req,
        cache_meta: Option<CacheMeta>,
    ) -> Result<Resp, SharedString> {
        if let Some(lock_handler) = self.lock_handler.as_ref() {
            let ret = lock_handler.handle(true).await;
            if !ret {
                //加锁操作，并且失败了
                if let Some(lock_error_handler) = self.lock_error_handler.as_ref() {
                    lock_error_handler.handle(()).await;
                }
                return Err(SharedString::from("lock failed before requesting"));
            }
            let result = self.call_with_loading(req, cache_meta).await;
            lock_handler.handle(false).await;
            return result;
        } else {
            return self.call_with_loading(req, cache_meta).await;
        }
    }

    async fn call_with_loading(
        &self,
        req: &Req,
        cache_meta: Option<CacheMeta>,
    ) -> Result<Resp, SharedString> {
        if self.show_loading {
            if let Some(loading_handler) = self.loading_handler.clone().or_else(|| {
                LazyLock::force(&DEFAULT_LOADING_HANDLER)
                    .read()
                    .unwrap()
                    .clone()
            }) {
                loading_handler.handle(true).await;
                let result = self.try_call(req, cache_meta).await;
                loading_handler.handle(false).await;
                return result;
            } else {
                return self.try_call(req, cache_meta).await;
            }
        } else {
            return self.try_call(req, cache_meta).await;
        }
    }

    async fn try_call(
        &self,
        req: &Req,
        cache_meta: Option<CacheMeta>,
    ) -> Result<Resp, SharedString> {
        let req = serde_json::to_string(&req).map_err(|err| {
            log::error!("Failed to serialize request: {}", err);
            SharedString::from("Failed to serialize request.")
        })?;
        let http_requestor = if let Some(http_requestor) =
            self.http_requestor.clone().or_else(|| {
                LazyLock::force(&DEFAULT_HTTP_REQUESTOR)
                    .read()
                    .unwrap()
                    .clone()
            }) {
            http_requestor.clone()
        } else {
            return Err(SharedString::from("http requestor unimplemented"));
        };
        let url = self.url.clone();
        let req = SharedString::from(req);
        match try_max_times(
            async |_last_error| http_requestor.handle((url.clone(), req.clone())).await,
            self.max_times
                .unwrap_or_else(|| NonZeroU32::new(1).unwrap()),
            self.interval,
        )
        .await
        {
            Ok(resp) => {
                let full_resp = serde_json::from_str::<serde_json::Value>(&resp).map_err(
                    |err| -> SharedString {
                        log::error!("响应数据格式不正确：{}", err);
                        return SharedString::from("响应数据格式不正确");
                    },
                )?;
                if let Some(data_unwrapper) = self.data_unwrapper.clone().or_else(|| {
                    LazyLock::force(&DEFAULT_DATA_UNWRAPPER)
                        .read()
                        .unwrap()
                        .clone()
                }) {
                    match data_unwrapper.handle(full_resp).await {
                        Ok(resp) => {
                            if let Some(cache_meta) = cache_meta.as_ref() {
                                let cache_map = LazyLock::force(&CACHE_MAP);
                                let mut cache_map = cache_map.lock().await;
                                cache_map.insert(
                                    cache_meta.key.to_string(),
                                    CacheData {
                                        data: resp.clone(),
                                        time: Date::now(),
                                        timeout: cache_meta.ttl,
                                    },
                                );
                            }
                            match serde_json::from_value::<Resp>(resp) {
                                Ok(resp) => {
                                    return Ok(resp);
                                }
                                Err(err) => {
                                    let err_msg =
                                        SharedString::from(format!("响应数据格式不正确：{}", err));
                                    if let Some(unwrap_error_handler) =
                                        self.unwrap_error_handler.clone().or_else(|| {
                                            LazyLock::force(&DEFAULT_UNWRAP_ERROR_HANDLER)
                                                .read()
                                                .unwrap()
                                                .clone()
                                        })
                                    {
                                        unwrap_error_handler.handle(err_msg.clone()).await;
                                    }
                                    return Err(err_msg);
                                }
                            }
                        }
                        Err(err) => {
                            if let Some(unwrap_error_handler) =
                                self.unwrap_error_handler.clone().or_else(|| {
                                    LazyLock::force(&DEFAULT_UNWRAP_ERROR_HANDLER)
                                        .read()
                                        .unwrap()
                                        .clone()
                                })
                            {
                                unwrap_error_handler.handle(err.clone()).await;
                            }
                            return Err(err);
                        }
                    }
                } else {
                    if let Some(cache_meta) = cache_meta.as_ref() {
                        let cache_map = LazyLock::force(&CACHE_MAP);
                        let mut cache_map = cache_map.lock().await;
                        cache_map.insert(
                            cache_meta.key.to_string(),
                            CacheData {
                                data: full_resp.clone(),
                                time: Date::now(),
                                timeout: cache_meta.ttl,
                            },
                        );
                    }
                    return serde_json::from_value::<Resp>(full_resp).map_err(
                        |err| -> SharedString {
                            log::error!("响应数据格式不正确：{}", err);
                            return SharedString::from("响应数据格式不正确");
                        },
                    );
                }
            }
            Err(err) => {
                if let Some(req_error_handler) = self.req_error_handler.clone().or_else(|| {
                    LazyLock::force(&DEFAULT_REQ_ERROR_HANDLER)
                        .read()
                        .unwrap()
                        .clone()
                }) {
                    req_error_handler.handle(err.clone()).await;
                }
                return Err(err);
            }
        }
    }
}

pub trait ApiExt {
    type Input;
    type Output;
    fn http_requestor(
        &self,
        http_requestor: impl Handler<(SharedString, SharedString), Result<SharedString, SharedString>>
            + Send
            + Sync,
    ) -> Request<Self::Input, Self::Output>;
    fn data_unwrapper(
        &self,
        data_unwrapper: impl Handler<serde_json::Value, Result<serde_json::Value, SharedString>>
            + Send
            + Sync,
    ) -> Request<Self::Input, Self::Output>;
    fn lock_handler(
        &self,
        lock_handler: impl Handler<bool, bool>,
    ) -> Request<Self::Input, Self::Output>;
    fn loading_handler(
        &self,
        loading_handler: impl Handler<bool, ()> + Send + Sync,
    ) -> Request<Self::Input, Self::Output>;
    fn disable_loading(&self) -> Request<Self::Input, Self::Output>;
    fn validate_error_handler(
        &self,
        validate_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> Request<Self::Input, Self::Output>;
    fn lock_error_handler(
        &self,
        lock_error_handler: impl Handler<(), ()>,
    ) -> Request<Self::Input, Self::Output>;
    fn req_error_handler(
        &self,
        req_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> Request<Self::Input, Self::Output>;
    fn unwrap_error_handler(
        &self,
        unwrap_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> Request<Self::Input, Self::Output>;
    fn use_cache(&self) -> Request<Self::Input, Self::Output>;
    fn try_max_times(
        &self,
        max_times: u32,
        interval: Option<u32>,
    ) -> Request<Self::Input, Self::Output>;
    async fn call(&self, req: &Self::Input) -> Result<Self::Output, SharedString>;
}

impl<T> ApiExt for T
where
    T: Api,
    T::Input: Serialize + 'static,
    T::Output: DeserializeOwned + 'static,
{
    type Input = T::Input;
    type Output = T::Output;

    fn http_requestor(
        &self,
        http_requestor: impl Handler<(SharedString, SharedString), Result<SharedString, SharedString>>
            + Send
            + Sync,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.http_requestor = Some(Arc::new(http_requestor));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn data_unwrapper(
        &self,
        data_unwrapper: impl Handler<serde_json::Value, Result<serde_json::Value, SharedString>>
            + Send
            + Sync,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.data_unwrapper = Some(Arc::new(data_unwrapper));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn lock_handler(&self, lock_handler: impl Handler<bool, bool>) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.lock_handler = Some(Arc::new(lock_handler));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn loading_handler(
        &self,
        loading_handler: impl Handler<bool, ()> + Send + Sync,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.loading_handler = Some(Arc::new(loading_handler));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn disable_loading(&self) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.show_loading = false;
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn validate_error_handler(
        &self,
        validate_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.validate_error_handler = Some(Arc::new(validate_error_handler));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn lock_error_handler(
        &self,
        lock_error_handler: impl Handler<(), ()>,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.lock_error_handler = Some(Arc::new(lock_error_handler));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn req_error_handler(
        &self,
        req_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.req_error_handler = Some(Arc::new(req_error_handler));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }
    fn unwrap_error_handler(
        &self,
        unwrap_error_handler: impl Handler<SharedString, ()> + Send + Sync,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.unwrap_error_handler = Some(Arc::new(unwrap_error_handler));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }

    fn use_cache(&self) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request
            .get_cache_meta
            .replace(Box::new(move |req: &T::Input| T::get_cache_meta(req)));
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }

    fn try_max_times(&self, max_times: u32, interval: Option<u32>) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        if Self::retryable() {
            request.try_max_times(max_times, interval);
        }
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request
    }

    async fn call(&self, req: &Self::Input) -> Result<Self::Output, SharedString> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.request_validator(|req: &T::Input| {
            T::validate_input(req).map_err(|error| SharedString::from(error.to_string()))
        });
        request.call(req).await
    }
}

pub static DEFAULT_HTTP_REQUESTOR: LazyLock<RwLock<Option<Arc<HttpRequestor>>>> =
    LazyLock::new(|| RwLock::new(None));
pub static DEFAULT_LOADING_HANDLER: LazyLock<RwLock<Option<Arc<LoadingHandler>>>> =
    LazyLock::new(|| RwLock::new(None));
pub static DEFAULT_DATA_UNWRAPPER: LazyLock<RwLock<Option<Arc<DataUnwrapper>>>> =
    LazyLock::new(|| RwLock::new(None));
pub static DEFAULT_VALIDATE_ERROR_HANDLER: LazyLock<RwLock<Option<Arc<ErrorHandler>>>> =
    LazyLock::new(|| RwLock::new(None));
pub static DEFAULT_REQ_ERROR_HANDLER: LazyLock<RwLock<Option<Arc<ErrorHandler>>>> =
    LazyLock::new(|| RwLock::new(None));
pub static DEFAULT_UNWRAP_ERROR_HANDLER: LazyLock<RwLock<Option<Arc<ErrorHandler>>>> =
    LazyLock::new(|| RwLock::new(None));
