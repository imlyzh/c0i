use serde::Serialize;
use std::collections::HashMap;
use xjbutil::value::Value;

pub type GValue = Value;
pub type DataMap = HashMap<String, GValue>;

#[derive(Debug, Serialize)]
pub struct DataCollection(HashMap<usize, DataMap>);

impl DataCollection {
    pub fn new() -> Self {
        DataCollection(HashMap::new())
    }

    pub fn insert<T>(
        &mut self,
        ptr: &T,
        key: impl Into<String>,
        value: impl Into<GValue>
    ) {
        let key = key.into();
        let value = value.into();

        #[cfg(any(debug_assertions, test))]
        eprintln!("INSERT: key0 = {:x?}, key1 = `{}`, value = {:?}", ptr as *const _, key, value);

        self.0.entry(ptr as *const _ as usize)
            .or_insert_with(DataMap::new)
            .insert(key, value);
    }

    pub fn insert_raw_key(
        &mut self,
        key0: usize,
        key1: impl Into<String>,
        value: impl Into<GValue>
    ) {
        let key1 = key1.into();
        let value = value.into();

        #[cfg(any(debug_assertions, test))]
        eprintln!("INSERT: key0 = {:?}, key1 = `{}`, value = {:?}", key0, key1, value);

        self.0.entry(key0)
            .or_insert_with(DataMap::new)
            .insert(key1, value);
    }

    pub fn get<T>(
        &self,
        ptr: &T,
        key: impl Into<String>
    ) -> Option<&GValue> {
        let key = key.into();

        #[cfg(any(debug_assertions, test))]
        eprintln!("GET: key0 = {:x?}, key1 = `{}`", ptr as *const _, key);

        self.0.get(&(ptr as *const _ as usize))
            .and_then(|map| map.get(&key))
    }

    pub fn get_raw_key(
        &self,
        key0: usize,
        key1: impl Into<String>
    ) -> Option<&GValue> {
        let key1 = key1.into();

        #[cfg(any(debug_assertions, test))]
        eprintln!("GET: key0 = {:?}, key1 = `{}`", key0, key1);

        self.0.get(&key0)
            .and_then(|map| map.get(&key1))
    }

    pub fn get_mut<T>(
        &mut self,
        ptr: &T,
        key: impl Into<String>
    ) -> Option<&mut GValue> {
        let key = key.into();

        #[cfg(any(debug_assertions, test))]
        eprintln!("GET: key0 = {:x?}, key1 = `{}`", ptr as *const _, key);

        self.0.get_mut(&(ptr as *const _ as usize))
            .and_then(|map| map.get_mut(&key))
    }

    pub fn get_mut_raw_key(
        &mut self,
        key0: usize,
        key1: impl Into<String>
    ) -> Option<&mut GValue> {
        let key1 = key1.into();

        #[cfg(any(debug_assertions, test))]
        eprintln!("GET: key0 = {:?}, key1 = `{}`", key0, key1);

        self.0.get_mut(&key0)
            .and_then(|map| map.get_mut(&key1))
    }
}
