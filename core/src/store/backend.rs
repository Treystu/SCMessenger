// Storage abstraction for cross-platform persistence

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Unified storage trait for cross-platform data persistence
pub trait StorageBackend: Send + Sync {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String>;
    fn remove(&self, key: &[u8]) -> Result<(), String>;
    fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String>;
    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String>;
    fn flush(&self) -> Result<(), String>;
}

/// In-memory storage useful for testing and temporary WASM execution
#[derive(Clone)]
pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl StorageBackend for MemoryStorage {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.data
            .write()
            .unwrap()
            .insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        Ok(self.data.read().unwrap().get(key).cloned())
    }

    fn remove(&self, key: &[u8]) -> Result<(), String> {
        self.data.write().unwrap().remove(key);
        Ok(())
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String> {
        let mut results = Vec::new();
        for (key, value) in self.data.read().unwrap().iter() {
            if key.starts_with(prefix) {
                results.push((key.clone(), value.clone()));
            }
        }
        Ok(results)
    }

    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String> {
        let count = self
            .data
            .read()
            .unwrap()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .count();
        Ok(count)
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct SledStorage {
    db: sled::Db,
}

#[cfg(not(target_arch = "wasm32"))]
impl SledStorage {
    pub fn new(path: &str) -> std::result::Result<Self, String> {
        let db = sled::open(path).map_err(|e| e.to_string())?;
        Ok(Self { db })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl StorageBackend for SledStorage {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.db.insert(key, value).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let value = self.db.get(key).map_err(|e| e.to_string())?;
        Ok(value.map(|ivec| ivec.to_vec()))
    }

    fn remove(&self, key: &[u8]) -> Result<(), String> {
        self.db.remove(key).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String> {
        let mut results = Vec::new();
        for item in self.db.scan_prefix(prefix) {
            let (k, v) = item.map_err(|e| e.to_string())?;
            results.push((k.to_vec(), v.to_vec()));
        }
        Ok(results)
    }

    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String> {
        Ok(self.db.scan_prefix(prefix).count())
    }

    fn flush(&self) -> Result<(), String> {
        self.db.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct IndexedDbStorage {
    db_name: String,
    store_name: String,
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

#[cfg(target_arch = "wasm32")]
impl IndexedDbStorage {
    pub async fn new(db_name: &str) -> std::result::Result<Self, String> {
        use js_sys::wasm_bindgen::JsCast;
        use rexie::*;
        let store_name = "scmessenger_store";

        let rexie = Rexie::builder(db_name)
            .version(1)
            .add_object_store(ObjectStore::new(store_name))
            .build()
            .await
            .map_err(|e| e.to_string())?;

        let data = Arc::new(RwLock::new(HashMap::new()));

        let transaction = rexie
            .transaction(&[store_name], TransactionMode::ReadOnly)
            .map_err(|e| e.to_string())?;
        let store = transaction.store(store_name).map_err(|e| e.to_string())?;

        let all_keys_js = store
            .get_all_keys(None, None)
            .await
            .map_err(|e| format!("{:?}", e))?;

        let mut map = data.write().unwrap();
        for key_js in all_keys_js {
            if let Ok(Some(value_js)) = store.get(key_js.clone()).await {
                if value_js.is_instance_of::<js_sys::Uint8Array>() {
                    let key_arr = js_sys::Uint8Array::new(&key_js);
                    let val_arr = js_sys::Uint8Array::new(&value_js);
                    map.insert(key_arr.to_vec(), val_arr.to_vec());
                }
            }
        }

        Ok(Self {
            db_name: db_name.to_string(),
            store_name: store_name.to_string(),
            data: data.clone(),
        })
    }

    fn persist_put(&self, key: Vec<u8>, value: Vec<u8>) {
        let db_name = self.db_name.clone();
        let store_name = self.store_name.clone();

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(rexie) = rexie::Rexie::builder(&db_name).version(1).build().await {
                if let Ok(tx) = rexie.transaction(&[&store_name], rexie::TransactionMode::ReadWrite)
                {
                    if let Ok(store) = tx.store(&store_name) {
                        let key_js = js_sys::Uint8Array::from(key.as_slice());
                        let value_js = js_sys::Uint8Array::from(value.as_slice());
                        // idb / rexie put
                        let _ = store.put(&value_js, Some(&key_js)).await;
                    }
                    let _ = tx.done().await;
                }
            }
        });
    }

    fn persist_remove(&self, key: Vec<u8>) {
        let db_name = self.db_name.clone();
        let store_name = self.store_name.clone();

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(rexie) = rexie::Rexie::builder(&db_name).version(1).build().await {
                if let Ok(tx) = rexie.transaction(&[&store_name], rexie::TransactionMode::ReadWrite)
                {
                    if let Ok(store) = tx.store(&store_name) {
                        let key_js = js_sys::Uint8Array::from(key.as_slice());
                        let _ = store.delete((&key_js).into()).await;
                    }
                    let _ = tx.done().await;
                }
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
impl StorageBackend for IndexedDbStorage {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.data
            .write()
            .unwrap()
            .insert(key.to_vec(), value.to_vec());
        self.persist_put(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        Ok(self.data.read().unwrap().get(key).cloned())
    }

    fn remove(&self, key: &[u8]) -> Result<(), String> {
        self.data.write().unwrap().remove(key);
        self.persist_remove(key.to_vec());
        Ok(())
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String> {
        let mut results = Vec::new();
        for (k, v) in self.data.read().unwrap().iter() {
            if k.starts_with(prefix) {
                results.push((k.clone(), v.clone()));
            }
        }
        Ok(results)
    }

    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String> {
        let count = self
            .data
            .read()
            .unwrap()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .count();
        Ok(count)
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }
}
