//! fontations.wasm - Modern Rust font processing library for WebAssembly
//! 
//! This library provides high-performance, memory-safe font processing capabilities
//! for web applications through WebAssembly. Built on the fontations Rust library,
//! it offers superior performance and safety compared to traditional C-based font
//! libraries while providing seamless JavaScript integration.
//!
//! ## Features
//! 
//! - **Memory Safety**: Rust's ownership system prevents buffer overflows and memory leaks
//! - **High Performance**: Zero-cost abstractions and optional SIMD acceleration
//! - **WASM-Native**: Designed specifically for web environments with async support
//! - **Modern Font Formats**: Full support for OpenType, TrueType, and variable fonts
//! - **JavaScript Integration**: Seamless interop with web applications
//!
//! ## Usage
//!
//! ```javascript
//! import init, { FontProcessor } from './pkg/fontations.js';
//!
//! async function main() {
//!     await init();
//!     
//!     const processor = new FontProcessor();
//!     const fontData = new Uint8Array(fontBytes);
//!     const fontInfo = processor.load_font(fontData, "my-font");
//!     
//!     console.log(`Loaded font: ${fontInfo.family}`);
//! }
//! ```

#![deny(missing_docs, unsafe_code)]
#![warn(clippy::all)]

pub mod wasm_api;

// Re-export main API for easier access
pub use wasm_api::*;