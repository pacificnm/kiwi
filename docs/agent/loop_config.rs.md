```rust
/// Configuration for the agent's loop.
use std::time::{Duration, Instant};

pub struct AgentLoopConfig {
    /// Maximum number of iterations allowed in a single loop run.
    pub max_iterations: u32,
    /// Duration to wait between each iteration if no new tasks are available.
    pub idle_duration: Duration,
    /// Flag indicating whether the agent is currently running in a debug mode.
    pub debug_mode: bool,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            idle_duration: Duration::from_secs(1),
            debug_mode: false,
        }
    }
}
```