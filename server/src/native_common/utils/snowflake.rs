use chrono::Utc;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

pub struct Snowflake {
    epoch: i64,
    worker_id: AtomicI64,
    sequence: AtomicI64,
    time: Arc<Mutex<i64>>,
}

impl Snowflake {
    pub fn default(epoch: Option<i64>) -> Snowflake {
        Snowflake::new(0, epoch)
    }

    pub fn new(worker_id: i64, epoch: Option<i64>) -> Snowflake {
        Snowflake {
            epoch: epoch.unwrap_or(1_609_430_400_000),
            worker_id: AtomicI64::new(worker_id),
            sequence: AtomicI64::new(0),
            time: Arc::new(Mutex::new(0)),
        }
    }

    pub fn set_worker_id(&self, worker_id: i64) -> &Self {
        self.worker_id.store(worker_id, Ordering::Relaxed);
        self
    }

    pub fn generate(&self) -> i64 {
        let mut last_timestamp = self.time.lock();
        let mut timestamp = self.get_time();
        if timestamp == *last_timestamp {
            let sequence = (self.sequence.load(Ordering::Relaxed) + 1) & (-1 ^ (-1 << 12));
            self.sequence.store(sequence, Ordering::Relaxed);
            if 0 == sequence && timestamp <= *last_timestamp {
                timestamp = self.get_time();
            }
        } else {
            self.sequence.store(0, Ordering::Relaxed);
        }
        *last_timestamp = timestamp;
        (timestamp << 22)
            | (self.worker_id.load(Ordering::Relaxed) << 12)
            | self.sequence.load(Ordering::Relaxed)
    }

    fn get_time(&self) -> i64 {
        Utc::now().timestamp_millis() - self.epoch
    }
}

pub struct LazySnowflake {
    sequence: AtomicI64,
    worker_id: AtomicI64,
}

impl LazySnowflake {
    pub fn default(epoch: Option<i64>) -> LazySnowflake {
        LazySnowflake::new(0, epoch)
    }

    pub fn new(worker_id: i64, epoch: Option<i64>) -> LazySnowflake {
        let time = Utc::now().timestamp_millis() - epoch.unwrap_or(1_609_430_400_000); // 2021-01-01
        LazySnowflake {
            sequence: AtomicI64::new(time << 12),
            worker_id: AtomicI64::new(worker_id),
        }
    }

    pub fn set_worker_id(&self, worker_id: i64) -> &Self {
        self.worker_id.store(worker_id, Ordering::Relaxed);
        self
    }

    pub fn generate(&self) -> i64 {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        (sequence << 10) | self.worker_id.load(Ordering::Relaxed)
    }
}
