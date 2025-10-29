//! WASM plugin loader using wasmtime
//!
//! Provides sandboxed plugin execution via WebAssembly

use super::{Capability, Plugin};
use crate::error::{Error, PluginError, Result};
use std::path::Path;
use wasmtime::*;

/// WASM plugin wrapper
pub struct WasmPlugin {
    name: String,
    version: String,
    capabilities: Vec<Capability>,
    instance: Option<Instance>,
    store: Option<Store<PluginState>>,
}

/// Plugin state accessible to WASM
struct PluginState {
    capabilities: Vec<Capability>,
}

impl WasmPlugin {
    /// Create a new WASM plugin from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Create wasmtime engine with default config
        let engine = Engine::default();

        // Load the WASM module
        let module = Module::from_file(&engine, path)
            .map_err(|e| Error::Plugin(PluginError::LoadFailed(e.to_string())))?;

        // Create a new store with plugin state
        let mut store = Store::new(
            &engine,
            PluginState {
                capabilities: vec![],
            },
        );

        // Define imports that plugins can use
        let mut linker = Linker::new(&engine);

        // Add logging function
        linker
            .func_wrap(
                "env",
                "log",
                |mut caller: Caller<'_, PluginState>, ptr: i32, len: i32| {
                    // Read string from WASM memory
                    let mem = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => return,
                    };

                    let data = mem.data(&caller);
                    if ptr < 0 || len < 0 || (ptr as usize + len as usize) > data.len() {
                        return;
                    }

                    if let Ok(message) =
                        std::str::from_utf8(&data[ptr as usize..(ptr + len) as usize])
                    {
                        println!("[WASM Plugin] {}", message);
                    }
                },
            )
            .map_err(|e| Error::Plugin(PluginError::LoadFailed(e.to_string())))?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| Error::Plugin(PluginError::LoadFailed(e.to_string())))?;

        // Extract plugin metadata (if exported)
        let name =
            Self::read_export_string(&instance, &mut store, "plugin_name").unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

        let version = Self::read_export_string(&instance, &mut store, "plugin_version")
            .unwrap_or_else(|| "0.1.0".to_string());

        Ok(Self {
            name,
            version,
            capabilities: vec![],
            instance: Some(instance),
            store: Some(store),
        })
    }

    /// Read a string export from WASM
    fn read_export_string(
        instance: &Instance,
        store: &mut Store<PluginState>,
        export_name: &str,
    ) -> Option<String> {
        // Try to get the exported function that returns string metadata
        let func = instance.get_func(&mut *store, export_name)?;

        let mut results = [Val::I32(0), Val::I32(0)];
        func.call(&mut *store, &[], &mut results).ok()?;

        // Extract pointer and length
        let ptr = results[0].unwrap_i32();
        let len = results[1].unwrap_i32();

        // Read from memory
        let memory = instance.get_memory(&mut *store, "memory")?;
        let data = memory.data(&*store);

        if ptr < 0 || len < 0 || (ptr as usize + len as usize) > data.len() {
            return None;
        }

        String::from_utf8(data[ptr as usize..(ptr + len) as usize].to_vec()).ok()
    }

    /// Call a WASM function
    fn call_function(&mut self, func_name: &str) -> Result<()> {
        let instance = self
            .instance
            .as_ref()
            .ok_or_else(|| Error::Plugin(PluginError::LoadFailed("No instance".into())))?;

        let store = self
            .store
            .as_mut()
            .ok_or_else(|| Error::Plugin(PluginError::LoadFailed("No store".into())))?;

        if let Some(func) = instance.get_func(&mut *store, func_name) {
            func.call(&mut *store, &[], &mut [])
                .map_err(|e| Error::Plugin(PluginError::ExecutionFailed(e.to_string())))?;
        }

        Ok(())
    }
}

impl Plugin for WasmPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn required_capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }

    fn init(&mut self) -> Result<()> {
        self.call_function("plugin_init")
    }

    fn cleanup(&mut self) -> Result<()> {
        self.call_function("plugin_cleanup")
    }
}

/// WASM plugin loader
pub struct WasmPluginLoader {
    search_paths: Vec<std::path::PathBuf>,
}

impl WasmPluginLoader {
    /// Create a new WASM plugin loader
    pub fn new() -> Self {
        Self {
            search_paths: vec![
                std::path::PathBuf::from("plugins"),
                std::path::PathBuf::from("./plugins"),
            ],
        }
    }

    /// Add a search path for plugins
    pub fn add_search_path(&mut self, path: impl Into<std::path::PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Load a specific plugin by path
    pub fn load(&self, path: impl AsRef<Path>) -> Result<WasmPlugin> {
        WasmPlugin::from_file(path)
    }

    /// Load all WASM plugins from search paths
    pub fn load_all(&self) -> Result<Vec<WasmPlugin>> {
        let mut plugins = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            let entries = std::fs::read_dir(search_path)
                .map_err(|e| Error::Plugin(PluginError::LoadFailed(e.to_string())))?;

            for entry in entries {
                let entry =
                    entry.map_err(|e| Error::Plugin(PluginError::LoadFailed(e.to_string())))?;
                let path = entry.path();

                // Only load .wasm files
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    match self.load(&path) {
                        Ok(plugin) => {
                            println!(
                                "Loaded WASM plugin: {} v{}",
                                plugin.name(),
                                plugin.version()
                            );
                            plugins.push(plugin);
                        }
                        Err(e) => {
                            eprintln!("Failed to load {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }
}

impl Default for WasmPluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_loader_creation() {
        let loader = WasmPluginLoader::new();
        assert!(!loader.search_paths.is_empty());
    }

    #[test]
    fn test_add_search_path() {
        let mut loader = WasmPluginLoader::new();
        loader.add_search_path("/custom/path");
        assert!(loader.search_paths.len() >= 3);
    }

    // Note: Actual WASM loading tests require .wasm files
    // These would be integration tests with sample plugins
}
