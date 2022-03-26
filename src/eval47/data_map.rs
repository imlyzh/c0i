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

    pub fn get<T>(
        &self,
        ptr: &T,
        key: impl Into<String>
    ) -> Option<&GValue> {
        self.0.get(&(ptr as *const _ as usize))
            .and_then(|map| map.get(&key.into()))
    }

    pub fn get_mut<T>(
        &mut self,
        ptr: &T,
        key: impl Into<String>
    ) -> Option<&mut GValue> {
        self.0.get_mut(&(ptr as *const _ as usize))
            .and_then(|map| map.get_mut(&key.into()))
    }
}
