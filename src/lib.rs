// OLE Reader
mod ole;

// Outlook Email Message File Parser
mod parser;
pub use parser::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn jsonFromUint8Array(slice: &[u8]) -> String {
    match Outlook::from_slice(slice) {
        Ok(outlook) => outlook.to_json().unwrap(),
        Err(_) => String::from("{\"error\": true}"),
    }
}
