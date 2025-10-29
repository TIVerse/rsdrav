// Example Rhai plugin for rsdrav
// This demonstrates basic plugin functionality

const PLUGIN_NAME = "hello_plugin";
const PLUGIN_VERSION = "1.0.0";

// Initialize the plugin
fn init() {
    log("Hello plugin initialized!");
}

// Cleanup when plugin is unloaded
fn cleanup() {
    log("Hello plugin cleaned up!");
}

// Custom function that can be called from the framework
fn greet(name) {
    "Hello from plugin, " + name + "!"
}

// Calculate something
fn calculate(a, b) {
    a + b * 2
}

// Process data
fn process_data(input) {
    log("Processing: " + input);
    input.to_upper()
}
