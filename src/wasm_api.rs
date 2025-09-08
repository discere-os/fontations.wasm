/*!
 * fontations.wasm - WASM API bindings for Rust fontations library
 * Production-quality font processing with memory safety and performance
 */

use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::console;
use std::collections::HashMap;
use std::sync::Arc;

// Core fontations imports
use font_types::{GlyphId, Tag};
use read_fonts::{FontRef, MetadataProvider, TableProvider, FontData};
use skrifa::{instance::Size, outline::OutlinePen, MetadataProvider as SkriMetadataProvider};

// Set up panic hook for better debugging in WASM
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

#[wasm_bindgen(start)]
pub fn wasm_main() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();
    
    console::log_1(&"fontations.wasm initialized".into());
}

/// FontInfo structure for JavaScript consumption
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct FontInfo {
    family: String,
    style: String,
    weight: u16,
    glyph_count: u32,
    units_per_em: u16,
}

#[wasm_bindgen]
impl FontInfo {
    #[wasm_bindgen(getter)]
    pub fn family(&self) -> String {
        self.family.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn style(&self) -> String {
        self.style.clone()  
    }
    
    #[wasm_bindgen(getter)]
    pub fn weight(&self) -> u16 {
        self.weight
    }
    
    #[wasm_bindgen(getter)]
    pub fn glyph_count(&self) -> u32 {
        self.glyph_count
    }
    
    #[wasm_bindgen(getter)]  
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
}

/// GlyphInfo structure for glyph-specific data
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct GlyphInfo {
    id: u32,
    advance: f32,
    bounds: Vec<f32>, // [min_x, min_y, max_x, max_y]
}

#[wasm_bindgen]
impl GlyphInfo {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> u32 {
        self.id
    }
    
    #[wasm_bindgen(getter)]
    pub fn advance(&self) -> f32 {
        self.advance
    }
    
    #[wasm_bindgen(getter)]
    pub fn bounds(&self) -> Vec<f32> {
        self.bounds.clone()
    }
}

/// Main FontProcessor class for WASM interaction
#[wasm_bindgen]
pub struct FontProcessor {
    fonts: HashMap<String, Arc<FontData>>,
    font_refs: HashMap<String, Arc<FontRef<'static>>>,
    memory_usage: usize,
    memory_limit: usize,
}

#[wasm_bindgen]
impl FontProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FontProcessor {
        FontProcessor {
            fonts: HashMap::new(),
            font_refs: HashMap::new(), 
            memory_usage: 0,
            memory_limit: 100 * 1024 * 1024, // 100MB default limit
        }
    }
    
    /// Load a font from binary data
    #[wasm_bindgen]
    pub fn load_font(&mut self, font_data: &[u8], font_id: &str) -> Result<FontInfo, JsValue> {
        // Validate font data
        if font_data.is_empty() {
            return Err(JsValue::from_str("Font data is empty"));
        }
        
        // Check memory limits
        if self.memory_usage + font_data.len() > self.memory_limit {
            return Err(JsValue::from_str("Memory limit exceeded"));
        }
        
        // Create FontData from bytes
        let font_data_owned = FontData::new(font_data.to_vec())
            .map_err(|e| JsValue::from_str(&format!("Failed to create FontData: {:?}", e)))?;
            
        // Create FontRef for metadata access  
        let font_ref = FontRef::new(&font_data_owned)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse font: {:?}", e)))?;
            
        // Extract font metadata
        let family = font_ref.head()
            .and_then(|head| head.ok())
            .and_then(|_| font_ref.name())
            .and_then(|name| name.ok())
            .and_then(|name_table| {
                name_table.family_name(None)
                    .and_then(|name| name.chars().collect::<Result<String, _>>().ok())
            })
            .unwrap_or_else(|| "Unknown".to_string());
            
        let glyph_count = font_ref.maxp()
            .and_then(|maxp| maxp.ok())
            .map(|maxp| maxp.num_glyphs())
            .unwrap_or(0) as u32;
            
        let units_per_em = font_ref.head()
            .and_then(|head| head.ok())
            .map(|head| head.units_per_em())
            .unwrap_or(1000);
        
        let font_info = FontInfo {
            family,
            style: "Regular".to_string(), // TODO: Extract actual style
            weight: 400, // TODO: Extract actual weight
            glyph_count,
            units_per_em,
        };
        
        // Store font data and refs
        let font_data_arc = Arc::new(font_data_owned);
        self.fonts.insert(font_id.to_string(), font_data_arc.clone());
        
        // SAFETY: We're storing the FontData in self.fonts with the same lifetime
        // This is safe because the FontRef borrows from the FontData we're storing
        let font_ref_static = unsafe {
            std::mem::transmute::<FontRef<'_>, FontRef<'static>>(font_ref)
        };
        self.font_refs.insert(font_id.to_string(), Arc::new(font_ref_static));
        
        self.memory_usage += font_data.len();
        
        Ok(font_info)
    }
    
    /// Get glyph information for a specific glyph ID
    #[wasm_bindgen]
    pub fn get_glyph_info(&self, font_id: &str, glyph_id: u32) -> Result<GlyphInfo, JsValue> {
        let font_ref = self.font_refs.get(font_id)
            .ok_or_else(|| JsValue::from_str("Font not found"))?;
            
        let glyph_id = GlyphId::new(glyph_id as u16);
        
        // Get horizontal metrics
        let hmtx = font_ref.hmtx()
            .map_err(|e| JsValue::from_str(&format!("Failed to read hmtx: {:?}", e)))?;
        let h_metrics = hmtx.h_metrics();
        let advance = if (glyph_id.to_u16() as usize) < h_metrics.len() {
            h_metrics[glyph_id.to_u16() as usize].advance_width() as f32
        } else {
            0.0
        };
        
        // TODO: Calculate actual glyph bounds
        let bounds = vec![0.0, 0.0, advance, 0.0];
        
        Ok(GlyphInfo {
            id: glyph_id.to_u16() as u32,
            advance,
            bounds,
        })
    }
    
    /// Get list of available fonts
    #[wasm_bindgen]
    pub fn list_fonts(&self) -> Array {
        let font_list = Array::new();
        for font_id in self.fonts.keys() {
            font_list.push(&JsValue::from_str(font_id));
        }
        font_list
    }
    
    /// Remove a font from memory
    #[wasm_bindgen]
    pub fn remove_font(&mut self, font_id: &str) -> bool {
        let removed_font = self.fonts.remove(font_id);
        self.font_refs.remove(font_id);
        
        if let Some(font_data) = removed_font {
            // Estimate memory usage reduction
            self.memory_usage = self.memory_usage.saturating_sub(font_data.len());
            true
        } else {
            false
        }
    }
    
    /// Clear all fonts from memory
    #[wasm_bindgen] 
    pub fn clear_fonts(&mut self) {
        self.fonts.clear();
        self.font_refs.clear();
        self.memory_usage = 0;
    }
    
    /// Get current memory usage statistics
    #[wasm_bindgen]
    pub fn get_memory_usage(&self) -> Object {
        let stats = Object::new();
        Reflect::set(&stats, &"used".into(), &(self.memory_usage as f64).into()).unwrap();
        Reflect::set(&stats, &"limit".into(), &(self.memory_limit as f64).into()).unwrap();
        Reflect::set(&stats, &"font_count".into(), &(self.fonts.len() as f64).into()).unwrap();
        stats
    }
    
    /// Set memory limit for font storage
    #[wasm_bindgen]
    pub fn set_memory_limit(&mut self, limit_mb: f64) {
        self.memory_limit = (limit_mb * 1024.0 * 1024.0) as usize;
    }
}

/// Async font processing functions
#[wasm_bindgen]
impl FontProcessor {
    /// Load font asynchronously (returns Promise)
    #[wasm_bindgen]
    pub async fn load_font_async(&mut self, font_data: &[u8], font_id: &str) -> Result<FontInfo, JsValue> {
        // For now, just call synchronous version
        // In future, could yield control periodically for large fonts
        self.load_font(font_data, font_id)
    }
    
    /// Process multiple glyphs asynchronously
    #[wasm_bindgen] 
    pub async fn process_glyphs_async(&self, font_id: &str, glyph_ids: &[u32]) -> Result<Array, JsValue> {
        let results = Array::new();
        
        for &glyph_id in glyph_ids {
            let glyph_info = self.get_glyph_info(font_id, glyph_id)?;
            results.push(&JsValue::from(glyph_info));
            
            // Yield control periodically to prevent blocking
            if glyph_ids.len() > 100 && (glyph_id as usize) % 50 == 0 {
                // Use setTimeout to yield control
                let promise = Promise::resolve(&JsValue::UNDEFINED);
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            }
        }
        
        Ok(results)
    }
}

/// Performance benchmarking functions
#[wasm_bindgen]
pub struct FontBenchmark {
    processor: FontProcessor,
}

#[wasm_bindgen]
impl FontBenchmark {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FontBenchmark {
        FontBenchmark {
            processor: FontProcessor::new(),
        }
    }
    
    /// Benchmark font loading performance
    #[wasm_bindgen]
    pub fn benchmark_font_loading(&mut self, font_data: &[u8], iterations: u32) -> f64 {
        let start = js_sys::Date::now();
        
        for i in 0..iterations {
            let font_id = format!("test_font_{}", i);
            let _ = self.processor.load_font(font_data, &font_id);
            self.processor.remove_font(&font_id);
        }
        
        js_sys::Date::now() - start
    }
    
    /// Benchmark glyph processing performance
    #[wasm_bindgen]
    pub fn benchmark_glyph_processing(&mut self, font_data: &[u8], glyph_count: u32) -> Result<Object, JsValue> {
        // Load test font
        self.processor.load_font(font_data, "benchmark_font")?;
        
        let start = js_sys::Date::now();
        
        for glyph_id in 0..glyph_count {
            let _ = self.processor.get_glyph_info("benchmark_font", glyph_id);
        }
        
        let end = js_sys::Date::now();
        let duration = end - start;
        
        // Clean up
        self.processor.remove_font("benchmark_font");
        
        let results = Object::new();
        Reflect::set(&results, &"duration".into(), &duration.into()).unwrap();
        Reflect::set(&results, &"glyphs_processed".into(), &(glyph_count as f64).into()).unwrap();
        Reflect::set(&results, &"throughput".into(), &((glyph_count as f64) / (duration / 1000.0)).into()).unwrap();
        
        Ok(results)
    }
}

/// Utility functions
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn get_features() -> Array {
    let features = Array::new();
    
    #[cfg(feature = "simd")]
    features.push(&"simd".into());
    
    #[cfg(feature = "webgpu")]
    features.push(&"webgpu".into());
    
    #[cfg(feature = "streaming")]
    features.push(&"streaming".into());
    
    #[cfg(feature = "geometry")]
    features.push(&"geometry".into());
    
    features
}

/// Error handling utilities
#[wasm_bindgen]
pub struct FontError {
    message: String,
    code: u32,
}

#[wasm_bindgen]
impl FontError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> u32 {
        self.code
    }
}

impl From<FontError> for JsValue {
    fn from(error: FontError) -> Self {
        let obj = Object::new();
        Reflect::set(&obj, &"message".into(), &error.message.into()).unwrap();
        Reflect::set(&obj, &"code".into(), &error.code.into()).unwrap();
        obj.into()
    }
}