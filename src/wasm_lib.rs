use wasm_bindgen::prelude::*;
use nostr_sdk::prelude::*;
use js_sys::Promise;
use web_sys::console;

/// WASM-compatible VectorBot wrapper
#[wasm_bindgen]
pub struct WasmVectorBot {
    keys: Keys,
    client: Client,
}

#[wasm_bindgen]
impl WasmVectorBot {
    /// Create a new VectorBot with default metadata
    #[wasm_bindgen(constructor)]
    pub fn new() -> Promise {
        let future = async {
            // Generate keys
            let keys = Keys::generate();

            // Create a simple client without MLS
            let mut client = Client::new(&keys);

            // Add default relays
            client.add_relay("wss://relay.damus.io").await.expect("Failed to add relay");
            client.add_relay("wss://nos.lol").await.expect("Failed to add relay");
            client.add_relay("wss://relay.nostr.band").await.expect("Failed to add relay");

            // Connect to relays
            client.connect().await;

            Ok::<WasmVectorBot, JsValue>(WasmVectorBot {
                keys,
                client,
            })
        };

        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Get the bot's public key as npub
    #[wasm_bindgen]
    pub fn get_public_key(&self) -> String {
        self.keys.public_key().to_bech32()
    }

    /// Send a private message to a recipient
    #[wasm_bindgen]
    pub fn send_private_message(&self, recipient_npub: String, message: String) -> Promise {
        let future = async move {
            let recipient = match PublicKey::from_bech32(&recipient_npub) {
                Ok(pk) => pk,
                Err(e) => {
                    console::error_1(&format!("Invalid recipient npub: {}", e).into());
                    return Err(JsValue::from_str(&format!("Invalid recipient npub: {}", e)));
                }
            };

            // Send private message
            match self.client.send_private_msg(recipient, &message, []).await {
                Ok(_) => Ok(JsValue::from_str("Message sent successfully")),
                Err(e) => {
                    console::error_1(&format!("Failed to send message: {:?}", e).into());
                    Err(JsValue::from_str(&format!("Failed to send message: {:?}", e)))
                }
            }
        };

        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Send a support ticket to admin
    #[wasm_bindgen]
    pub fn send_support_ticket(&self, message: String) -> Promise {
        // Admin npub from requirements
        let admin_npub = "npub132lq2gvwx9ae3wug5hy7a5tcs48jamynfsuact2cvgjavs5uk8vqeme4sy";

        let future = async move {
            let recipient = match PublicKey::from_bech32(admin_npub) {
                Ok(pk) => pk,
                Err(e) => {
                    console::error_1(&format!("Invalid admin npub: {}", e).into());
                    return Err(JsValue::from_str(&format!("Invalid admin npub: {}", e)));
                }
            };

            // Send private message
            match self.client.send_private_msg(recipient, &message, []).await {
                Ok(_) => Ok(JsValue::from_str("Support ticket sent successfully")),
                Err(e) => {
                    console::error_1(&format!("Failed to send support ticket: {:?}", e).into());
                    Err(JsValue::from_str(&format!("Failed to send support ticket: {:?}", e)))
                }
            }
        };

        wasm_bindgen_futures::future_to_promise(future)
    }
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn start() {
    console::log_1(&"WASM module initialized".into());
}