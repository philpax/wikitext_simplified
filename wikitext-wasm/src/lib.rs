use wasm_bindgen::prelude::*;
use wikitext_simplified::{parse_and_simplify_wikitext, Spanned, WikitextSimplifiedNode};
use wikitext_util::wikipedia_pwt_configuration;

/// Parse wikitext and return the simplified AST as JSON
#[wasm_bindgen]
pub fn parse_wikitext(wikitext: &str) -> Result<JsValue, JsValue> {
    let config = wikipedia_pwt_configuration();

    match parse_and_simplify_wikitext(wikitext, &config) {
        Ok(nodes) => {
            serde_wasm_bindgen::to_value(&nodes)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&format!("{}", e))),
    }
}

/// Result type for parsing that includes both the AST and any warnings
#[derive(serde::Serialize)]
pub struct ParseResult {
    pub nodes: Vec<Spanned<WikitextSimplifiedNode>>,
    pub success: bool,
}

/// Parse wikitext and return a structured result
#[wasm_bindgen]
pub fn parse_wikitext_with_result(wikitext: &str) -> JsValue {
    let config = wikipedia_pwt_configuration();

    match parse_and_simplify_wikitext(wikitext, &config) {
        Ok(nodes) => {
            let result = ParseResult {
                nodes,
                success: true,
            };
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
        Err(e) => {
            // Return error as a JS object
            let error_obj = js_sys::Object::new();
            js_sys::Reflect::set(&error_obj, &"success".into(), &false.into()).ok();
            js_sys::Reflect::set(&error_obj, &"error".into(), &format!("{}", e).into()).ok();
            error_obj.into()
        }
    }
}
