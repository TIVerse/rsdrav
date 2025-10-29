//! Example demonstrating WASM plugin loading
//!
//! Run with: cargo run --example wasm_plugin_demo --features plugin-wasm

#[cfg(feature = "plugin-wasm")]
fn main() -> rsdrav::Result<()> {
    use rsdrav::plugin::{PluginManager, WasmPluginLoader};
    use rsdrav::prelude::*;

    println!("=== WASM Plugin Demo ===\n");

    // Create plugin loader
    let loader = WasmPluginLoader::new();

    // Try to load plugins from plugins/ directory
    println!("Loading WASM plugins from: plugins/");
    match loader.load_all() {
        Ok(plugins) => {
            if plugins.is_empty() {
                println!("No WASM plugins found.");
                println!("\nTo create a WASM plugin:");
                println!("1. Create a Rust library with cdylib target");
                println!("2. Compile to WASM: cargo build --target wasm32-unknown-unknown");
                println!("3. Place .wasm file in plugins/ directory");
            } else {
                println!("Loaded {} plugin(s):\n", plugins.len());

                // Register plugins with manager
                let mut manager = PluginManager::new();
                for mut plugin in plugins {
                    println!("  - {} v{}", plugin.name(), plugin.version());
                    println!("    Capabilities: {:?}", plugin.required_capabilities());

                    // Initialize plugin
                    match plugin.init() {
                        Ok(_) => println!("    ✓ Initialized"),
                        Err(e) => println!("    ✗ Failed to initialize: {}", e),
                    }

                    // Register with manager
                    if let Err(e) = manager.register(Box::new(plugin)) {
                        println!("    ✗ Failed to register: {}", e);
                    }
                }

                println!(
                    "\nTotal registered plugins: {}",
                    manager.list_plugins().len()
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to load plugins: {}", e);
        }
    }

    Ok(())
}

#[cfg(not(feature = "plugin-wasm"))]
fn main() {
    eprintln!("This example requires the 'plugin-wasm' feature.");
    eprintln!("Run with: cargo run --example wasm_plugin_demo --features plugin-wasm");
}
