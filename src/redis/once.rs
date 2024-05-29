use crate::redis::types::Generic;
use redis::{Client, Commands};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::from_str;
use std::ops::Deref;

/// A struct which provides a way to execute a function only and exactly once.
pub struct Once<T> {
    data: Generic<T>,
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

impl<T> Once<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a new `Once` instance.
    /// This is a slower version, because it connects to redis and store a default value, if nothing is there.
    /// The name has to be unique and the same in all your services, because it is used as a key in redis.
    pub fn new(name: &str, client: Client) -> Self {
        let data = Generic::new(name, client.clone());
        Self {
            data,
            already_called: OnceState::Unknown,
        }
    }

    /// Creates a new `Once` instance.
    /// This is a faster version, because it does not connect to redis, but you have to provide a data object.
    pub fn take_data(data: Generic<T>) -> Self {
        Self {
            data,
            already_called: OnceState::Unknown,
        }
    }

    /// Check the completion state of the function through redis.
    /// But this is a slower version, because it connects to redis and store a default value, if nothing is there.
    /// If you want to check the completion state without connecting to redis, use [Once::is_completed].
    /// Mostly used when you used [Once::take_data].
    pub fn check_completion_state(&mut self) -> bool {
        // Pushes a value to the cache if it is not already there.
        let val = self.data.client.exists(&self.data.key).unwrap();
        self.already_called = match val {
            true => OnceState::Called,
            false => OnceState::NotCalled,
        };
        val
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
    /// If there is already a value stored in redis, it pulls this value.
    /// So if there is a value already in place, it needs a separate call to get the value.
    /// ```
    /// use dtypes::redis::sync::Once;
    /// use redis::Client;
    ///
    /// let client = Client::open("redis://localhost:6379").unwrap();
    /// let mut once: Once<i32> = Once::new("test_once_simple", client.clone());
    /// assert_eq!(once.check_completion_state(), false);
    ///
    /// // Do some calculations
    /// let state = once.call_once(|| {
    ///     let d;
    ///     // calculations...
    ///     d = 35;
    ///     d
    /// }).unwrap();
    ///
    /// assert_eq!(state, &35);
    /// assert_eq!(once.check_completion_state(), true);
    /// assert_eq!(once.cached(), Some(&35));
    /// ```
    pub fn call_once<F>(&mut self, f: F) -> Result<&T, OnceError>
    where
        F: FnOnce() -> T,
    {
        if self.is_completed() {
            return Err(OnceError::AlreadyCalled);
        }

        let call_val = f();

        let val = serde_json::to_string(&call_val).expect("Failed to serialize value");
        let res: Option<String> =
            self.data
                .client
                .set_nx(&self.data.key, val)
                .unwrap_or_else(|_| {
                    self.data
                        .get_conn()
                        .get(&self.data.key)
                        .expect("Failed to get value")
                });
        let v = res.unwrap();
        let parsed_val = from_str(&v).unwrap();
        self.data.cache = Some(parsed_val);

        self.already_called = OnceState::Called;

        Ok(self.data.acquire())
    }
}

impl<T> Deref for Once<T> {
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_once_parallel() {
        use crate::redis::sync::Once;
        use redis::Client;

        let client = Client::open("redis://localhost:6379").unwrap();
        let mut once: Once<i32> = Once::new("test_once_parallel", client.clone());
        let mut once2: Once<i32> = Once::new("test_once_parallel", client.clone());

        let state = once
            .call_once(|| {
                let d;
                // calculations...
                d = 35;
                d
            })
            .unwrap();

        let state2 = once2
            .call_once(|| {
                let d;
                // calculations...
                d = 1;
                d
            })
            .unwrap();

        assert_eq!(state, &35);
        assert_eq!(state2, &35);
        assert_eq!(once.cached().unwrap(), &35);
    }
}
