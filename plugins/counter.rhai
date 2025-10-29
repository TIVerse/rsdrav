// Counter Plugin
// Demonstrates stateful plugin with persistent data

const PLUGIN_NAME = "counter_plugin";

let count = 0;

fn init() {
    count = 0;
    log("Counter plugin initialized with count = 0");
}

fn increment() {
    count += 1;
    log("Count incremented to: " + count);
    count
}

fn decrement() {
    count -= 1;
    log("Count decremented to: " + count);
    count
}

fn get_count() {
    count
}

fn reset() {
    count = 0;
    log("Counter reset!");
}

fn cleanup() {
    log("Counter final value: " + count);
}
