use crate::ArcFn;
use futures::channel::oneshot;
use js_sys::Date;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

pub type Watcher<D> = ArcFn<D, ()>;

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

struct FnHandler<In, Out>(Rc<dyn Fn(In) -> Pin<Box<dyn Future<Output = Out>>> + 'static>);
unsafe impl<In, Out> Send for FnHandler<In, Out> {}
unsafe impl<In, Out> Sync for FnHandler<In, Out> {}

pub struct CacheMgr<In, Out, E> {
    handler: FnHandler<In, Result<(Out, Option<u64>), E>>,
    input: In,
    cache: RwLock<Option<Result<CacheData<Out>, E>>>,
    waiting_tasks: Mutex<(bool, Vec<oneshot::Sender<Result<Out, E>>>)>,
    watchers: Arc<RwLock<Vec<Watcher<Out>>>>,
    result_watchers: Arc<RwLock<Vec<Watcher<Result<Out, E>>>>>,
    phantom1: PhantomData<In>,
    phantom2: PhantomData<Out>,
    phantom3: PhantomData<E>,
}

impl<In, Out, E> CacheMgr<In, Out, E>
where
    In: Clone + 'static,
    Out: Clone + Debug + 'static,
    E: Clone + Debug + 'static,
{
    pub fn new<H, Fut>(handler: H, input: In) -> CacheMgr<In, Out, E>
    where
        Fut: Future<Output = Result<(Out, Option<u64>), E>> + 'static,
        H: Fn(In) -> Fut + 'static,
    {
        let handler = Rc::new(handler);
        CacheMgr {
            handler: FnHandler(Rc::new(move |input: In| {
                let handler = handler.clone();
                Box::pin(async move { handler(input).await })
            })),
            input: input,
            cache: RwLock::new(None),
            waiting_tasks: Mutex::new((false, Vec::new())),
            watchers: Default::default(),
            result_watchers: Default::default(),
            phantom1: PhantomData,
            phantom2: PhantomData,
            phantom3: PhantomData,
        }
    }
    fn get_fresh_cache(&self) -> Option<Out> {
        let cache = self.cache.read().unwrap();
        let cache_data = cache
            .as_ref()
            .map(|cache| {
                match cache {
                    Ok(cache) => {
                        if cache.expired() {
                            //缓存已过期
                            None
                        } else {
                            //新鲜的缓存数据
                            Some(cache.data.clone())
                        }
                    }
                    Err(_err) => {
                        //上一次请求是失败的
                        None
                    }
                }
            })
            .flatten();
        drop(cache);
        return cache_data;
    }
    pub async fn get_fresh_data(&self) -> Result<Out, E> {
        if let Some(cache_data) = self.get_fresh_cache() {
            return Ok(cache_data);
        } else {
            return self.get_or_wait_data().await;
        }
    }
    async fn get_or_wait_data(&self) -> Result<Out, E> {
        let mut waiting_tasks = self.waiting_tasks.lock().unwrap();
        if waiting_tasks.0 {
            //有任务在运行
            let (sender, receiver) = oneshot::channel::<Result<Out, E>>();
            waiting_tasks.1.push(sender);
            drop(waiting_tasks);
            let result = receiver.await.unwrap();
            return result;
        } else {
            waiting_tasks.0 = true;
            drop(waiting_tasks);
            let result = (self.handler.0)(self.input.clone()).await;
            self.cache
                .write()
                .unwrap()
                .replace(result.clone().map(|(out, timeout)| CacheData {
                    data: out,
                    time: Date::now(),
                    timeout: timeout,
                }));
            let result = result.map(|(out, _timeout)| out);
            let mut waiting_tasks = self.waiting_tasks.lock().unwrap();
            let tasks: Vec<_> = waiting_tasks.1.drain(..).collect();
            waiting_tasks.0 = false;
            drop(waiting_tasks);
            for task in tasks {
                task.send(result.clone()).unwrap();
            }
            for result_watcher in self.result_watchers.read().unwrap().iter() {
                result_watcher(result.clone());
            }
            if let Ok(out) = &result {
                for watcher in self.watchers.read().unwrap().iter() {
                    watcher(out.clone());
                }
            }
            return result;
        }
    }
    pub async fn clear_cache(&self) {
        self.cache.write().unwrap().take();
    }

    pub fn watch(&self, watcher: impl Fn(Out) + Send + Sync + 'static) -> impl Fn() {
        let watcher = ArcFn::from(watcher);
        let mut watchers = self.watchers.write().unwrap();
        if watchers.iter().all(|item| item != &watcher) {
            watchers.push(watcher.clone());
        }
        drop(watchers);
        if let Some(cache) = self.get_fresh_cache() {
            watcher(cache);
        }
        let watchers = self.watchers.clone();
        return move || {
            watchers.write().unwrap().retain(|item| item != &watcher);
        };
    }

    pub fn watch_result(
        &self,
        result_watcher: impl Fn(Result<Out, E>) + Send + Sync + 'static,
    ) -> impl Fn() {
        let result_watcher = ArcFn::from(result_watcher);
        let mut result_watchers = self.result_watchers.write().unwrap();
        if result_watchers.iter().all(|item| item != &result_watcher) {
            result_watchers.push(result_watcher.clone());
        }
        drop(result_watchers);
        let cache = self.cache.read().unwrap();
        if let Some(cache) = cache.as_ref() {
            match cache {
                Ok(cache) => {
                    if cache.expired() {
                        //缓存已过期
                    } else {
                        //新鲜的缓存数据
                        result_watcher(Ok(cache.data.clone()));
                    }
                }
                Err(err) => {
                    //上一次请求是失败的
                    result_watcher(Err(err.clone()));
                }
            }
        }
        drop(cache);
        let result_watchers = self.result_watchers.clone();
        return move || {
            result_watchers
                .write()
                .unwrap()
                .retain(|item| item != &result_watcher);
        };
    }
}
