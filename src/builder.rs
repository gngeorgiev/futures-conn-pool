use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use futures::Future;
use parking_lot::RwLock;
use tokio::io::Result;

use crate::backoff::BackoffStrategy;
use crate::factory::ObjectFactory;
use crate::object::PoolObject;
use crate::pool::Pool;

pub struct PoolBuilder<T>
where
    T: PoolObject,
{
    _factory: Option<Arc<ObjectFactory<T>>>,
    _timeout: Option<Duration>,
    _max_tries: Option<usize>,
    _capacity: Option<usize>,
    _backoff: BackoffStrategy,
}

impl<T> PoolBuilder<T>
where
    T: PoolObject,
{
    pub fn new() -> PoolBuilder<T> {
        PoolBuilder {
            _factory: None,
            _timeout: Some(Duration::from_secs(10)),
            _max_tries: Some(10),
            _capacity: None,
            _backoff: BackoffStrategy::None,
        }
    }

    pub fn factory<F>(mut self, factory: impl Fn() -> F + Send + Sync + 'static) -> Self
    where
        F: Future<Output = Result<T>> + 'static,
    {
        self._factory = Some(Arc::new(move || Box::pin(factory())));
        self
    }

    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self._timeout = timeout;
        self
    }

    pub fn max_tries(mut self, max_tries: Option<usize>) -> Self {
        self._max_tries = max_tries;
        self
    }

    pub fn capacity(mut self, capacity: Option<usize>) -> Self {
        self._capacity = capacity;
        self
    }

    pub fn backoff(mut self, backoff: BackoffStrategy) -> Self {
        self._backoff = backoff;
        self
    }

    pub fn build(self) -> Pool<T> {
        Pool {
            factory: self._factory.expect("A pool connector is required"),
            objects: Arc::new(RwLock::new(VecDeque::with_capacity(
                self._capacity.unwrap_or_else(|| 10),
            ))),
            backoff: self._backoff,
            timeout: self._timeout,
            max_tries: self._max_tries,
            capacity: self._capacity,
        }
    }
}
