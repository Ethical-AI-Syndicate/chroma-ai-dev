//! Memory profiling utilities for ChromaAI Dev
//!
//! This module provides memory profiling capabilities for detecting and debugging
//! memory issues in the application.
//!
//! # Usage
//!
//! ```rust
//! use chroma_ai_dev::memory_profiler;
//!
//! // Take a memory snapshot
//! memory_profiler::snapshot("before_operation");
//!
//! // Do some work
//! do_work();
//!
//! // Take another snapshot and compare
//! memory_profiler::snapshot("after_operation");
//! memory_profiler::print_comparison();
//! ```

use std::collections::VecDeque;

/// Maximum number of memory samples to keep for trend analysis
const MAX_SAMPLES: usize = 100;

/// Memory snapshot containing statistics
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub label: String,
    pub allocated_bytes: u64,
}

impl MemorySnapshot {
    #[cfg(not(target_os = "unknown"))]
    fn current(label: &str) -> Self {
        // Try to get memory info - on systems without /proc, this will return 0
        let allocated = std::thread::spawn(|| {
            #[cfg(not(target_os = "unknown"))]
            {
                // On Linux, try to read from /proc/self/statm
                std::fs::read_to_string("/proc/self/statm")
                    .ok()
                    .and_then(|s| {
                        let parts: Vec<&str> = s.split_whitespace().collect();
                        if parts.len() >= 2 {
                            // Page size * resident pages
                            let pages: u64 = parts[1].parse().ok()?;
                            Some(pages * 4096) // Typically 4KB pages
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0)
            }
            #[cfg(target_os = "unknown")]
            {
                0
            }
        })
        .join()
        .unwrap_or(0);

        Self {
            label: label.to_string(),
            allocated_bytes: allocated,
        }
    }

    #[cfg(target_os = "unknown")]
    fn current(label: &str) -> Self {
        Self {
            label: label.to_string(),
            allocated_bytes: 0,
        }
    }
}

/// Memory profiler for tracking memory usage over time
#[derive(Default)]
pub struct Profiler {
    snapshots: VecDeque<MemorySnapshot>,
}

impl Profiler {
    /// Take a memory snapshot with the given label
    pub fn snapshot(&mut self, label: &str) {
        let snapshot = MemorySnapshot::current(label);

        // Remove oldest if at capacity
        if self.snapshots.len() >= MAX_SAMPLES {
            self.snapshots.pop_front();
        }

        self.snapshots.push_back(snapshot);
    }

    /// Print comparison between first and last snapshot
    pub fn print_comparison(&self) {
        if self.snapshots.len() < 2 {
            println!("Need at least 2 snapshots to compare");
            return;
        }

        let first = self.snapshots.front().unwrap();
        let last = self.snapshots.back().unwrap();

        if first.allocated_bytes > 0 && last.allocated_bytes > 0 {
            let diff = last.allocated_bytes as i64 - first.allocated_bytes as i64;
            println!(
                "=== Memory Comparison: {} -> {} ===",
                first.label, last.label
            );
            println!("Memory: {} bytes ({:+.1} KB)", diff, diff as f64 / 1024.0);
        }
    }

    /// Check for potential memory leaks (growth trend)
    pub fn check_for_leak(&self) -> bool {
        if self.snapshots.len() < 5 {
            return false;
        }

        let samples: Vec<_> = self.snapshots.iter().collect();
        let first_half: u64 = samples[..samples.len() / 2]
            .iter()
            .map(|s| s.allocated_bytes)
            .sum::<u64>()
            / (samples.len() / 2) as u64;
        let second_half: u64 = samples[samples.len() / 2..]
            .iter()
            .map(|s| s.allocated_bytes)
            .sum::<u64>()
            / (samples.len() - samples.len() / 2) as u64;

        // If second half average is 50% higher than first half, potential leak
        first_half > 0 && second_half > first_half * 150 / 100
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Get current memory usage (if available)
pub fn get_memory_usage() -> u64 {
    std::thread::spawn(|| {
        #[cfg(not(target_os = "unknown"))]
        {
            std::fs::read_to_string("/proc/self/statm")
                .ok()
                .and_then(|s| {
                    let parts: Vec<&str> = s.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let pages: u64 = parts[1].parse().ok()?;
                        Some(pages * 4096)
                    } else {
                        None
                    }
                })
                .unwrap_or(0)
        }
        #[cfg(target_os = "unknown")]
        {
            0
        }
    })
    .join()
    .unwrap_or(0)
}

/// Track allocations for a specific scope (no-op without feature)
#[macro_export]
macro_rules! memory_scope {
    ($name:expr, $block:block) => {{
        let _ = $name;
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }
}
