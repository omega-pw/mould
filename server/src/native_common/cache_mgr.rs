use futures::channel::oneshot;
use futures::lock::Mutex;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;
use std::time::Instant;
use tihu::Handler;
use tokio::sync::RwLock;

struct CacheData<D> {
    data: D,
    time: Instant,
    timeout: Duration,
}

impl<D> CacheData<D> {
    fn expired(&self) -> bool {
        self.time.elapsed() > self.timeout
    }
}

struct CacheMgr<H, In, Out, E> {
    handler: H,
    input: In,
    cache: RwLock<Option<CacheData<Out>>>,
    waiting_tasks: Mutex<(bool, Vec<oneshot::Sender<Result<Out, E>>>)>,
    phantom1: PhantomData<In>,
    phantom2: PhantomData<Out>,
    phantom3: PhantomData<E>,
}

impl<H, In, Out, E> CacheMgr<H, In, Out, E>
where
    H: Handler<In, Out = Result<(Out, Duration), E>>,
    In: Clone,
    Out: Clone + Debug,
    E: Clone + Debug,
{
    pub fn new(handler: H, input: In) -> CacheMgr<H, In, Out, E> {
        CacheMgr {
            handler: handler,
            input: input,
            cache: RwLock::new(None),
            waiting_tasks: Mutex::new((false, Vec::new())),
            phantom1: PhantomData,
            phantom2: PhantomData,
            phantom3: PhantomData,
        }
    }
    pub async fn get_fresh_data(&self) -> Result<Out, E> {
        let cache = self.cache.read().await;
        let cache_data = cache
            .as_ref()
            .map(|cache| {
                if cache.expired() {
                    //缓存已过期
                    None
                } else {
                    //新鲜的缓存数据
                    Some(cache.data.clone())
                }
            })
            .flatten();
        drop(cache);
        if let Some(cache_data) = cache_data {
            return Ok(cache_data);
        } else {
            return self.get_or_wait_data().await;
        }
    }
    async fn get_or_wait_data(&self) -> Result<Out, E> {
        let mut waiting_tasks = self.waiting_tasks.lock().await;
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
            let result = match self.handler.handle(self.input.clone()).await {
                Ok((out, timeout)) => {
                    self.cache.write().await.replace(CacheData {
                        data: out.clone(),
                        time: Instant::now(),
                        timeout: timeout,
                    });
                    Ok(out)
                }
                Err(err) => Err(err),
            };
            let mut waiting_tasks = self.waiting_tasks.lock().await;
            let tasks: Vec<_> = waiting_tasks.1.drain(..).collect();
            waiting_tasks.0 = false;
            drop(waiting_tasks);
            for task in tasks {
                task.send(result.clone()).unwrap();
            }
            return result;
        }
    }
    pub async fn clear_cache(&self) {
        self.cache.write().await.take();
    }
}
