/// 通用 LazyCache 宏
#[macro_export]
macro_rules! lazy_cache {
    ($name:ident : $key:ty => $value:ty) => {
        lazy_static! {
            static ref $name: parking_lot::RwLock<std::collections::HashMap<$key, std::sync::Arc<$value>>> =
                parking_lot::RwLock::new(std::collections::HashMap::new());
        }
    };
}
