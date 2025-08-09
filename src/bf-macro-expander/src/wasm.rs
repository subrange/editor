use wasm_bindgen::prelude::*;
use serde_wasm_bindgen;
use crate::{create_macro_expander, MacroExpanderOptions, MacroExpanderResult};

#[wasm_bindgen]
pub struct WasmMacroExpander {
    expander: crate::MacroExpander,
}

#[wasm_bindgen]
impl WasmMacroExpander {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            expander: create_macro_expander(),
        }
    }
    
    #[wasm_bindgen]
    pub fn expand(&mut self, input: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let options: MacroExpanderOptions = if options.is_undefined() || options.is_null() {
            MacroExpanderOptions::default()
        } else {
            serde_wasm_bindgen::from_value(options)
                .map_err(|e| JsValue::from_str(&format!("Invalid options: {}", e)))?
        };
        
        let result = self.expander.expand(input, options);
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    #[wasm_bindgen(js_name = expandWithDefaults)]
    pub fn expand_with_defaults(&mut self, input: &str) -> Result<JsValue, JsValue> {
        let result = self.expander.expand(input, MacroExpanderOptions::default());
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    #[wasm_bindgen(js_name = getExpandedCode)]
    pub fn get_expanded_code(&mut self, input: &str) -> String {
        let result = self.expander.expand(input, MacroExpanderOptions::default());
        result.expanded
    }
}

// Standalone functions for simpler API
#[wasm_bindgen]
pub fn expand_macros(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let mut expander = WasmMacroExpander::new();
    expander.expand(input, options)
}

#[wasm_bindgen]
pub fn expand_macros_simple(input: &str) -> String {
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    result.expanded
}

// For console error messages in the browser
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// Optional: Add a panic hook for better error messages
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}