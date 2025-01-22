use crate::LightString;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tihu::Api;

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

pub type HttpRequestor = dyn Handler<(LightString, LightString), Result<LightString, LightString>>;
//loading展示器
pub type LoadingHandler = dyn Handler<bool, ()>;
//锁处理器，lock参数为true表示加锁，lock参数为false表示解锁，返回true表示执行成功，返回false表示执行失败
pub type LockHandler = dyn Handler<bool, bool>;
//数据解包器，输入请求器返回的结果，输出Result<成功结果，业务错误>
pub type DataUnwrapper = dyn Handler<serde_json::Value, Result<serde_json::Value, LightString>>;
//错误处理器
pub type ErrorHandler = dyn Handler<LightString, ()>;
//加锁错误处理器
pub type LockErrorHandler = dyn Handler<(), ()>;

pub struct Request<Req, Resp> {
    url: LightString,
    method: Method,
    http_requestor: Option<Arc<HttpRequestor>>,
    show_loading: bool,
    loading_handler: Option<Arc<LoadingHandler>>,
    lock_handler: Option<Arc<LockHandler>>,
    data_unwrapper: Option<Arc<DataUnwrapper>>,
    lock_error_handler: Option<Arc<LockErrorHandler>>,
    req_error_handler: Option<Arc<ErrorHandler>>,
    unwrap_error_handler: Option<Arc<ErrorHandler>>,
    phantom1: PhantomData<Req>,
    phantom2: PhantomData<Resp>,
}

impl<Req, Resp> Request<Req, Resp>
where
    Req: Serialize + 'static,
    Resp: DeserializeOwned + 'static,
{
    pub fn new(url: LightString) -> Self {
        Self {
            url: url,
            method: Method::Post,
            http_requestor: None,
            show_loading: true,
            loading_handler: None,
            lock_handler: None,
            data_unwrapper: None,
            lock_error_handler: None,
            req_error_handler: None,
            unwrap_error_handler: None,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
    pub fn http_requestor(
        &mut self,
        http_requestor: impl Handler<(LightString, LightString), Result<LightString, LightString>>,
    ) -> &mut Self {
        self.http_requestor = Some(Arc::new(http_requestor));
        return self;
    }
    pub fn data_unwrapper(
        &mut self,
        data_unwrapper: impl Handler<serde_json::Value, Result<serde_json::Value, LightString>>,
    ) -> &mut Self {
        self.data_unwrapper = Some(Arc::new(data_unwrapper));
        return self;
    }
    pub fn lock_handler(&mut self, lock_handler: impl Handler<bool, bool>) -> &mut Self {
        self.lock_handler = Some(Arc::new(lock_handler));
        return self;
    }
    pub fn loading_handler(&mut self, loading_handler: impl Handler<bool, ()>) -> &mut Self {
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
        req_error_handler: impl Handler<LightString, ()>,
    ) -> &mut Self {
        self.req_error_handler = Some(Arc::new(req_error_handler));
        return self;
    }
    pub fn unwrap_error_handler(
        &mut self,
        unwrap_error_handler: impl Handler<LightString, ()>,
    ) -> &mut Self {
        self.unwrap_error_handler = Some(Arc::new(unwrap_error_handler));
        return self;
    }

    pub async fn call(&self, req: &Req) -> Result<Resp, LightString> {
        return self.call_with_lock(req).await;
    }

    async fn call_with_lock(&self, req: &Req) -> Result<Resp, LightString> {
        if let Some(lock_handler) = self.lock_handler.as_ref() {
            let ret = lock_handler.handle(true).await;
            if !ret {
                //加锁操作，并且失败了
                if let Some(lock_error_handler) = self.lock_error_handler.as_ref() {
                    lock_error_handler.handle(()).await;
                }
                return Err(LightString::from("lock failed before requesting"));
            }
            let result = self.call_with_loading(req).await;
            lock_handler.handle(false).await;
            return result;
        } else {
            return self.call_with_loading(req).await;
        }
    }

    async fn call_with_loading(&self, req: &Req) -> Result<Resp, LightString> {
        if self.show_loading {
            if let Some(loading_handler) = self
                .loading_handler
                .as_ref()
                .or_else(|| unsafe { DEFAULT_LOADING_HANDLER.as_ref() })
            {
                loading_handler.handle(true).await;
                let result = self.try_call(req).await;
                loading_handler.handle(false).await;
                return result;
            } else {
                return self.try_call(req).await;
            }
        } else {
            return self.try_call(req).await;
        }
    }

    async fn try_call(&self, req: &Req) -> Result<Resp, LightString> {
        let req = serde_json::to_string(&req).map_err(|err| {
            log::error!("Failed to serialize request: {}", err);
            LightString::from("Failed to serialize request.")
        })?;
        let http_requestor = if let Some(http_requestor) = self
            .http_requestor
            .as_ref()
            .or_else(|| unsafe { DEFAULT_HTTP_REQUESTOR.as_ref() })
        {
            http_requestor.clone()
        } else {
            return Err(LightString::from("http requestor unimplemented"));
        };
        match http_requestor.handle((self.url.clone(), req.into())).await {
            Ok(resp) => {
                let full_resp = serde_json::from_str::<serde_json::Value>(&resp).map_err(
                    |err| -> LightString {
                        log::error!("响应数据格式不正确：{}", err);
                        return LightString::Static("响应数据格式不正确");
                    },
                )?;
                if let Some(data_unwrapper) = self
                    .data_unwrapper
                    .as_ref()
                    .or_else(|| unsafe { DEFAULT_DATA_UNWRAPPER.as_ref() })
                {
                    match data_unwrapper.handle(full_resp).await {
                        Ok(resp) => match serde_json::from_value::<Resp>(resp) {
                            Ok(resp) => {
                                return Ok(resp);
                            }
                            Err(err) => {
                                let err_msg =
                                    LightString::from(format!("响应数据格式不正确：{}", err));
                                if let Some(unwrap_error_handler) = self
                                    .unwrap_error_handler
                                    .as_ref()
                                    .or_else(|| unsafe { DEFAULT_UNWRAP_ERROR_HANDLER.as_ref() })
                                {
                                    unwrap_error_handler.handle(err_msg.clone()).await;
                                }
                                return Err(err_msg);
                            }
                        },
                        Err(err) => {
                            if let Some(unwrap_error_handler) = self
                                .unwrap_error_handler
                                .as_ref()
                                .or_else(|| unsafe { DEFAULT_UNWRAP_ERROR_HANDLER.as_ref() })
                            {
                                unwrap_error_handler.handle(err.clone()).await;
                            }
                            return Err(err);
                        }
                    }
                } else {
                    return serde_json::from_value::<Resp>(full_resp).map_err(
                        |err| -> LightString {
                            log::error!("响应数据格式不正确：{}", err);
                            return LightString::Static("响应数据格式不正确");
                        },
                    );
                }
            }
            Err(err) => {
                if let Some(req_error_handler) = self
                    .req_error_handler
                    .as_ref()
                    .or_else(|| unsafe { DEFAULT_REQ_ERROR_HANDLER.as_ref() })
                {
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
        http_requestor: impl Handler<(LightString, LightString), Result<LightString, LightString>>,
    ) -> Request<Self::Input, Self::Output>;
    fn data_unwrapper(
        &self,
        data_unwrapper: impl Handler<serde_json::Value, Result<serde_json::Value, LightString>>,
    ) -> Request<Self::Input, Self::Output>;
    fn lock_handler(
        &self,
        lock_handler: impl Handler<bool, bool>,
    ) -> Request<Self::Input, Self::Output>;
    fn loading_handler(
        &self,
        loading_handler: impl Handler<bool, ()>,
    ) -> Request<Self::Input, Self::Output>;
    fn disable_loading(&self) -> Request<Self::Input, Self::Output>;
    fn lock_error_handler(
        &self,
        lock_error_handler: impl Handler<(), ()>,
    ) -> Request<Self::Input, Self::Output>;
    fn req_error_handler(
        &self,
        req_error_handler: impl Handler<LightString, ()>,
    ) -> Request<Self::Input, Self::Output>;
    fn unwrap_error_handler(
        &self,
        unwrap_error_handler: impl Handler<LightString, ()>,
    ) -> Request<Self::Input, Self::Output>;
    async fn call(&self, req: &Self::Input) -> Result<Self::Output, LightString>;
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
        http_requestor: impl Handler<(LightString, LightString), Result<LightString, LightString>>,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.http_requestor = Some(Arc::new(http_requestor));
        request
    }
    fn data_unwrapper(
        &self,
        data_unwrapper: impl Handler<serde_json::Value, Result<serde_json::Value, LightString>>,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.data_unwrapper = Some(Arc::new(data_unwrapper));
        request
    }
    fn lock_handler(&self, lock_handler: impl Handler<bool, bool>) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.lock_handler = Some(Arc::new(lock_handler));
        request
    }
    fn loading_handler(
        &self,
        loading_handler: impl Handler<bool, ()>,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.loading_handler = Some(Arc::new(loading_handler));
        request
    }
    fn disable_loading(&self) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.show_loading = false;
        request
    }
    fn lock_error_handler(
        &self,
        lock_error_handler: impl Handler<(), ()>,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.lock_error_handler = Some(Arc::new(lock_error_handler));
        request
    }
    fn req_error_handler(
        &self,
        req_error_handler: impl Handler<LightString, ()>,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.req_error_handler = Some(Arc::new(req_error_handler));
        request
    }
    fn unwrap_error_handler(
        &self,
        unwrap_error_handler: impl Handler<LightString, ()>,
    ) -> Request<T::Input, T::Output> {
        let mut request = Request::new(Self::namespace().to_string().into());
        request.unwrap_error_handler = Some(Arc::new(unwrap_error_handler));
        request
    }
    async fn call(&self, req: &Self::Input) -> Result<Self::Output, LightString> {
        let request = Request::new(Self::namespace().to_string().into());
        request.call(req).await
    }
}

pub static mut DEFAULT_HTTP_REQUESTOR: Option<Arc<HttpRequestor>> = None;
pub static mut DEFAULT_LOADING_HANDLER: Option<Arc<LoadingHandler>> = None;
pub static mut DEFAULT_DATA_UNWRAPPER: Option<Arc<DataUnwrapper>> = None;
pub static mut DEFAULT_REQ_ERROR_HANDLER: Option<Arc<ErrorHandler>> = None;
pub static mut DEFAULT_UNWRAP_ERROR_HANDLER: Option<Arc<ErrorHandler>> = None;
