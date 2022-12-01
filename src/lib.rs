mod commissions;
mod generator;
mod plan;
mod serializer;
#[cfg(test)]
mod tests;
mod utils;

use js_sys::{Array, JsString};
use scheduler::models::Code;
use wasm_bindgen::prelude::*;
mod api;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub enum Semester {
    First,
    Second,
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub struct SubjectInfo {
    code: Code,
    name: String,
    pub credits: u8,
}

#[wasm_bindgen]
impl SubjectInfo {
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "string[]")]
    pub type StringArray;

    #[wasm_bindgen(typescript_type = "[[string, string], [string, string]][]")]
    pub type CollisionExceptions;
}

impl From<StringArray> for Vec<String> {
    fn from(sa: StringArray) -> Self {
        Array::from(&sa)
            .iter()
            .map(|v| v.as_string().expect("Must be a string array"))
            .collect()
    }
}

impl From<Vec<String>> for StringArray {
    fn from(sa: Vec<String>) -> Self {
        //serde_wasm_bindgen::to_value::<Vec<String>>(&sa.into()).unwrap().into()
        JsValue::from(
            sa.into_iter()
                .map(|v| v.parse::<JsString>().unwrap())
                .collect::<Array>(),
        )
        .into()
    }
}
