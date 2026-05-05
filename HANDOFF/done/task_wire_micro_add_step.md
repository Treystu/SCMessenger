TARGET: core/src/dspy/modules.rs
WIRE: add_step(&mut self, step: &str) -- call it after dspy_create_cot() in iron_core.rs to append reasoning steps to the chain
VERIFY: cargo check --workspace