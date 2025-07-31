use nostr_library::VectorBot;
use nostr_sdk::prelude::*;
use std::error::Error;
use reqwest::Client;

/// Main function to demonstrate the usage of the VectorBot.
///
/// This function sets up a VectorBot, sends a private message to a master chat,
/// and handles notifications for gift wrap events.
///
/// # Returns
///
/// Result::Ok if the operation was successful, or Result::Err if an error occurred.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // A master npub for communicating in case of issues
    let master_npub = PublicKey::from_bech32("npub1f88ah0k9hp2puakdjt85ve6kzkmngjy2l6k0llkav9mamgprgg7stc3c45")?;

    // Generate new random keys
    let keys = Keys::generate();

    // Show bech32 public key
    let bech32_pubkey: String = keys.public_key().to_bech32()?;
    let bech32_private_key: String = keys.secret_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);
    println!("Bech32 PrivateKey: {}", bech32_private_key);

    // Create a new VectorBot with default metadata
    let bot = VectorBot::quick(keys).await;

    // Get a chat channel for the master public key
    let master_chat = bot.get_chat(master_npub).await;

    // Send a private message to the master chat
    let _response = master_chat.send_private_message("Bot now online").await;

    // Set up notification handler for gift wrap events
    bot.client.handle_notifications(|notification| {
        let bot_clone = bot.clone();

        async move {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind == Kind::GiftWrap {
                    match bot_clone.client.unwrap_gift_wrap(&event).await {
                        Ok(UnwrappedGift { rumor, sender }) => {
                            if rumor.kind == Kind::PrivateDirectMessage {
                                let content: String = match rumor.content.trim().to_lowercase().as_str() {
                                    "/rand" => rand::random::<u16>().to_string(),
                                    "/help" => help(),
                                    cmd if cmd.starts_with("/pivx") => {
                                        // Extract currency from command if provided
                                        let currency = if let Some(currency_part) = rumor.content.split_whitespace().nth(1) {
                                            currency_part.to_lowercase()
                                        } else {
                                            "usd".to_string() // Default to USD if no currency specified
                                        };

                                        // Fetch the cryptocurrency price
                                        let r_client = Client::new();
                                        let url = format!("https://pivxla.bz/oracle/api/v1/price/{}", currency);

                                        match r_client.get(&url).send().await {
                                            Ok(response) => {
                                                if response.status().is_success() {
                                                    if let Ok(json) = response.json::<serde_json::Value>().await {
                                                        if let Some(value) = json.get("value").and_then(|v| v.as_f64()) {
                                                            format!("PIVX price in {}: {:.8}", currency.to_uppercase(), value)
                                                        } else {
                                                            "Failed to parse PIVX price".to_string()
                                                        }
                                                    } else {
                                                        "Failed to parse PIVX price response".to_string()
                                                    }
                                                } else {
                                                    "Failed to fetch PIVX price".to_string()
                                                }
                                            }
                                            Err(_) => "Error fetching PIVX price".to_string(),
                                        }
                                    },
                                    "/cat" => {
                                        // Fetch the cat image from the URL
                                        let cat_url = "https://cataas.com/cat?json=true";
                                        let r_client = Client::new();

                                        match r_client.get(cat_url).send().await {
                                            Ok(response) => {
                                                if response.status().is_success() {
                                                    // Parse the JSON response to get the image URL
                                                    if let Ok(json) = response.json::<serde_json::Value>().await {
                                                        if let Some(image_url) = json.get("url").and_then(|url| url.as_str()) {
                                                            // Fetch the actual image
                                                            let image_response = r_client.get(image_url).send().await;

                                                            match image_response {
                                                                Ok(img_response) => {
                                                                    if img_response.status().is_success() {
                                                                        // Create an AttachmentFile with the image data
                                                                        let bytes = img_response.bytes().await.unwrap().to_vec();
                                                                        let extension = match json.get("mimetype").and_then(|mimetype| mimetype.as_str()) {
                                                                            // // Images
                                                                            // "png" => "image/png",
                                                                            // "jpg" | "jpeg" => "image/jpeg",
                                                                            // "gif" => "image/gif",
                                                                            // "webp" => "image/webp",
                                                                            Some("image/png") => "png",
                                                                            Some("image/jpeg") => "jpg",
                                                                            Some("image/gif") => "gif",
                                                                            Some("image/webp") => "webp",
                                                                            Some(&_) => "png",
                                                                            None => "png",
                                                                        };

                                                                        
                                                                        let attached_file = nostr_library::AttachmentFile {
                                                                            bytes,
                                                                            img_meta: None,
                                                                            extension: extension.to_string(),
                                                                        };

                                                                        //println!("Attached file: {:#?}", attached_file);

                                                                        // Send the image file
                                                                        let normal_chat = bot_clone.get_chat(sender).await;
                                                                        println!("chat channel created");
                                                                        let sendAttatched = normal_chat.send_private_file(Some(attached_file)).await;
                                                                        println!("AttatchedMessageSend: {:#?}", sendAttatched);
                                                                        "Here is your cat image!".to_string()
                                                                    } else {
                                                                        "Failed to fetch cat image".to_string()
                                                                    }
                                                                }
                                                                Err(_) => "Error fetching cat image".to_string(),
                                                            }
                                                        } else {
                                                            "Invalid cat image response".to_string()
                                                        }
                                                    } else {
                                                        "Failed to parse cat image response".to_string()
                                                    }
                                                } else {
                                                    "Failed to fetch cat image metadata".to_string()
                                                }
                                            }
                                            Err(_) => "Error fetching cat image".to_string(),
                                        }
                                    }
                                    _ => String::from(
                                        "Invalid command, send /help to see all commands.",
                                    ),
                                };

                                let normal_chat = bot_clone.get_chat(sender).await;
                                println!("Sending response: {:#?}", &content);
                                let _response = normal_chat.send_private_message(&content).await;
                            }
                        }
                        Err(e) => println!("Impossible to decrypt direct message: {e}"),
                    }
                }
            }
            Ok(false) // Set to true to exit from the loop
        }
    }).await?;

    Ok(())
}

/// Provides help information with available commands.
///
/// # Returns
///
/// A string containing the help information.
fn help() -> String {
    let mut output = String::new();
    output.push_str("Commands:\n");
    output.push_str("/rand - Random number\n");
    output.push_str("/help - Help\n");
    output.push_str("/cat - Get a random cat image\n");
    output.push_str("/pivx [currency] - Get PIVX price (default: USD)");
    output
}