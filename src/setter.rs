pub trait WallpaperSetter {
    fn set(&self, path: &str, args: &[String]) -> bool;
}

use std::collections::HashMap;

pub struct WallpaperSetterFactory {
    registered: HashMap<String, Box<dyn Fn() -> Box<dyn WallpaperSetter> + Send + Sync>>,
}

impl WallpaperSetterFactory {
    pub fn new() -> Self {
        WallpaperSetterFactory {
            registered: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Box<dyn WallpaperSetter> + Send + Sync + 'static,
    {
        self.registered.insert(name.to_string(), Box::new(factory));
    }

    pub fn get(&self, name: &str) -> Option<&(dyn Fn() -> Box<dyn WallpaperSetter> + Send + Sync)> {
        self.registered.get(name).map(Box::as_ref)
    }
}
