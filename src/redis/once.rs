use crate::redis::bool_type::TBool;
use crate::redis::sync::Mutex;
use redis::Client;

/// A struct which provides a way to execute a function only and exactly once.
pub struct Once {
    data: Mutex<bool>,
    conn: Option<redis::Connection>,
    already_called: OnceState,
}

#[derive(Debug)]
pub enum OnceError {
    /// The function has already been called.
    AlreadyCalled,
    /// The function has not been called yet.
    NotCalled,
}

pub enum OnceState {
    /// The function has already been called.
    Called,
    /// The function has not been called yet.
    NotCalled,
    /// The state is unknown.
    Unknown,
}

impl Once {
    /// Creates a new `Once` instance.
    /// This is a slower version, because it connects to redis and store a default value, if nothing is there.
    /// The name has to be unique and the same in all your services, because it is used as a key in redis.
    pub fn new(name: &str, client: Client) -> Self {
        let data = Mutex::new(TBool::with_value_default(false, name, client));
        Self {
            data,
            conn: None,
            already_called: OnceState::Unknown,
        }
    }

    /// Creates a new `Once` instance.
    /// This is a faster version, because it does not connect to redis, but you have to provide a data object.
    pub fn take_data(data: TBool) -> Self {
        let data = Mutex::new(data);
        Self {
            data,
            conn: None,
            already_called: OnceState::Unknown,
        }
    }

    /// Check the completion state of the function through redis.
    /// But this is a slower version, because it connects to redis and store a default value, if nothing is there.
    /// If you want to check the completion state without connecting to redis, use [Once::is_completed].
    /// Mostly used when you used [Once::take_data].
    /// It helps a lot, when multiple calculations are okay, but the final result should be stored only once.
    /// So you can check the completion state after each calculation.
    ///
    /// ```
    /// use dtypes::redis::sync::Once;
    /// use dtypes::redis::sync::Mutex;
    /// use dtypes::redis::types::Di32 as i32;
    /// use redis::Client;
    ///
    /// let client = Client::open("redis://localhost:6379").unwrap();
    /// let mut once = Once::new("test", client.clone());
    /// let mut data = Mutex::new(i32::with_value(1, "test_val", client));
    /// let mut val = None;
    /// assert_eq!(once.check_completion_state(), false);
    ///
    /// // Do some calculations
    /// let state = once.call_once(|| {
    ///     let d;
    ///     // calculations...
    ///     d = 35;
    ///     val = Some(d);
    /// }).unwrap();
    ///
    /// {
    ///     let mut d = data.lock().unwrap();
    ///     // Locks the data object and we check the completion state.
    ///     // Otherwise we do not want to store it.
    ///     assert_eq!(state, false);
    ///     d.store(val.expect("Here should be a value")).expect("Failed to store value");
    /// }
    /// ```
    pub fn check_completion_state(&mut self) -> bool {
        // Pushes a value to the cache if it is not already there.
        let mut d = self.data.lock().unwrap();
        d.cache.unwrap_or_else(|| {
            d.try_get().unwrap_or_else(|| {
                d.store(false).expect("Failed to store value");
                false
            })
        })
    }

    /// Returns `true` if the function has already been called.
    /// Returns `false` in following situations:
    /// - The function has not been called yet.
    /// - The function has been called, but not completed yet.
    /// - The state is unknown, because there was no attempt to connect to redis.
    ///
    /// If you want to check the completion state within redis, use [Once::check_completion_state].
    pub fn is_completed(&self) -> bool {
        match self.already_called {
            OnceState::Called => true,
            OnceState::NotCalled | OnceState::Unknown => false,
        }
    }

    /// Calls the function only once.
    /// If the function has already been called, it returns an error.
    /// Otherwise it calls the function and stores the completion state.
    /// The completion state from redis will be returned in the Result, it is not your state.
    pub fn call_once<F>(&mut self, f: F) -> Result<bool, OnceError>
    where
        F: FnOnce(),
    {
        if self.is_completed() {
            return Err(OnceError::AlreadyCalled);
        }

        f();
        let prev_val = self.check_completion_state();
        self.data
            .lock()
            .unwrap()
            .store(true)
            .expect("Failed to store value");
        self.already_called = OnceState::Called;

        Ok(prev_val)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_once_parallel() {
        todo!()
    }
}
