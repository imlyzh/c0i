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
        self.0.entry(ptr as *const _ as usize)
            .or_insert_with(DataMap::new)
            .insert(key.into(), value.into());
    }

    pub fn insert_raw_key(
        &mut self,
        key0: usize,
        key1: impl Into<String>,
        value: impl Into<GValue>
    ) {
        self.0.entry(key0)
            .or_insert_with(DataMap::new)
            .insert(key1.into(), value.into());
    }

    pub fn get<T>(
        &self,
        ptr: &T,
        key: impl Into<String>
    ) -> Option<&GValue> {
        self.0.get(&(ptr as *const _ as usize))
            .and_then(|map| map.get(&key.into()))
    }

    pub fn get_raw_key(
        &self,
        key0: usize,
        key1: impl Into<String>
    ) -> Option<&GValue> {
        self.0.get(&key0)
            .and_then(|map| map.get(&key1.into()))
    }

    pub fn get_mut<T>(
        &mut self,
        ptr: &T,
        key: impl Into<String>
    ) -> Option<&mut GValue> {
        self.0.get_mut(&(ptr as *const _ as usize))
            .and_then(|map| map.get_mut(&key.into()))
    }

    pub fn get_mut_raw_key(
        &mut self,
        key0: usize,
        key1: impl Into<String>
    ) -> Option<&mut GValue> {
        self.0.get_mut(&key0)
            .and_then(|map| map.get_mut(&key1.into()))
    }
}
