use memory_pak_core::{
    MemoryPakApp, PersistedState, QueryInput, SetItemNotesInput, SetItemStatusInput,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmMemoryPak {
    app: MemoryPakApp,
}

#[wasm_bindgen]
impl WasmMemoryPak {
    #[wasm_bindgen(constructor)]
    pub fn new(state_json: Option<String>) -> Result<WasmMemoryPak, JsValue> {
        console_error_panic_hook::set_once();
        let state = match state_json {
            Some(json) if !json.trim().is_empty() => serde_json::from_str::<PersistedState>(&json)
                .map_err(|err| js_error(format!("Invalid persisted state: {err}")))?,
            _ => PersistedState::default(),
        };

        Ok(Self {
            app: MemoryPakApp::from_persisted_state(state),
        })
    }

    #[wasm_bindgen(js_name = loadInitialState)]
    pub fn load_initial_state(&self) -> Result<JsValue, JsValue> {
        to_js(self.app.initial_state())
    }

    #[wasm_bindgen(js_name = queryConsoles)]
    pub fn query_consoles(&self, input: JsValue) -> Result<JsValue, JsValue> {
        let input = from_js::<QueryInput>(input)?;
        to_js(self.app.query_consoles(input))
    }

    #[wasm_bindgen(js_name = queryGames)]
    pub fn query_games(&self, input: JsValue) -> Result<JsValue, JsValue> {
        let input = from_js::<QueryInput>(input)?;
        to_js(self.app.query_games(input))
    }

    #[wasm_bindgen(js_name = queryCollectibles)]
    pub fn query_collectibles(&self, input: JsValue) -> Result<JsValue, JsValue> {
        let input = from_js::<QueryInput>(input)?;
        to_js(self.app.query_collectibles(input))
    }

    #[wasm_bindgen(js_name = setItemStatus)]
    pub fn set_item_status(&mut self, input: JsValue) -> Result<JsValue, JsValue> {
        let input = from_js::<SetItemStatusInput>(input)?;
        let result = self
            .app
            .set_item_status(input)
            .map_err(|err| js_error(err.to_string()))?;
        to_js(result)
    }

    #[wasm_bindgen(js_name = setItemNotes)]
    pub fn set_item_notes(&mut self, input: JsValue) -> Result<JsValue, JsValue> {
        let input = from_js::<SetItemNotesInput>(input)?;
        let result = self
            .app
            .set_item_notes(input)
            .map_err(|err| js_error(err.to_string()))?;
        to_js(result)
    }

    #[wasm_bindgen(js_name = importJson)]
    pub fn import_json(&mut self, json: String) -> Result<JsValue, JsValue> {
        let stats = self
            .app
            .import_json(&json)
            .map_err(|err| js_error(err.to_string()))?;
        to_js(stats)
    }

    #[wasm_bindgen(js_name = exportJson)]
    pub fn export_json(&self) -> Result<String, JsValue> {
        self.app
            .export_json()
            .map_err(|err| js_error(format!("Failed to export JSON: {err}")))
    }

    #[wasm_bindgen(js_name = getCollectionStats)]
    pub fn get_collection_stats(&self) -> Result<JsValue, JsValue> {
        to_js(self.app.collection_stats())
    }

    #[wasm_bindgen(js_name = snapshotStateJson)]
    pub fn snapshot_state_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(self.app.persisted_state())
            .map_err(|err| js_error(format!("Failed to serialize state: {err}")))
    }
}

fn from_js<T>(value: JsValue) -> Result<T, JsValue>
where
    T: serde::de::DeserializeOwned,
{
    serde_wasm_bindgen::from_value(value).map_err(|err| js_error(err.to_string()))
}

fn to_js<T>(value: T) -> Result<JsValue, JsValue>
where
    T: Serialize,
{
    serde_wasm_bindgen::to_value(&value).map_err(|err| js_error(err.to_string()))
}

fn js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}
