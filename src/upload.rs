use log::debug;
use nostr_sdk::hashes::{sha256::Hash as Sha256Hash, Hash};
use nostr_sdk::{
    nips::nip96::{ServerConfig, UploadResponse, UploadResponseStatus},
    nips::nip98::{HttpData, HttpMethod},
    NostrSigner, TagKind, TagStandard, Url,
};
use reqwest::{
    multipart::{self, Part},
    Body, Client,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::mpsc;

/// Configuration options for the upload client
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Connection timeout duration
    pub connect_timeout: std::time::Duration,
    /// Idle pool timeout duration
    pub pool_idle_timeout: std::time::Duration,
    /// Maximum idle connections per host
    pub pool_max_idle_per_host: usize,
    /// Stall detection threshold (in milliseconds)
    pub stall_threshold: u32,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            connect_timeout: std::time::Duration::from_secs(5),
            pool_idle_timeout: std::time::Duration::from_secs(90),
            pool_max_idle_per_host: 2,
            stall_threshold: 200, // 20 seconds (200 * 100ms)
        }
    }
}

/// Errors that can occur during upload operations
#[derive(Error, Debug)]
pub enum UploadError {
    /// Reqwest client error
    #[error("Reqwest client error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Multipart MIME error
    #[error("Multipart MIME error")]
    MultipartMimeError,

    /// Upload error with message
    #[error("Upload error: {0}")]
    UploadError(String),

    /// Response decode error
    #[error("Failed to decode response")]
    ResponseDecodeError,

    /// Generic error with message
    #[error("{0}")]
    GenericError(String),
}

/// Makes a reqwest client with configurable settings
///
/// Creates a reqwest Client with optional proxy configuration and custom settings.
///
/// # Arguments
///
/// * `proxy` - Optional proxy address for the client.
/// * `config` - Optional configuration for the client.
///
/// # Returns
///
/// A Result containing the configured Client or an UploadError.
pub fn make_client(
    proxy: Option<SocketAddr>,
    config: Option<UploadConfig>,
) -> Result<Client, UploadError> {
    let config = config.unwrap_or_default();
    let client: Client = {
        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .pool_idle_timeout(config.pool_idle_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host);

        if let Some(proxy) = proxy {
            let proxy = format!("socks5h://{proxy}");
            use reqwest::Proxy;
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        builder.build()?
    };

    Ok(client)
}

/// Custom upload stream that allows tracking progress
///
/// This stream reads data in chunks and reports progress through a shared counter.
pub struct ProgressTrackingStream {
    bytes_sent: Arc<Mutex<u64>>,
    inner: mpsc::Receiver<Result<Vec<u8>, std::io::Error>>,
}

impl ProgressTrackingStream {
    /// Creates a new ProgressTrackingStream
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be sent through the stream
    /// * `bytes_sent` - Counter for tracking bytes sent
    /// * `chunk_size` - Size of each chunk to send (default: 64KB)
    ///
    /// # Returns
    ///
    /// A new ProgressTrackingStream
    pub fn new(data: Vec<u8>, bytes_sent: Arc<Mutex<u64>>, chunk_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(8); // Buffer size of 8 chunks

        // Spawn a background task to feed the stream
        tokio::spawn(async move {
            let chunk_size = chunk_size;
            let mut position = 0;

            while position < data.len() {
                let end = std::cmp::min(position + chunk_size, data.len());
                let chunk = data[position..end].to_vec();
                let chunk_size = chunk.len();

                // Send chunk through channel
                if tx.send(Ok(chunk)).await.is_err() {
                    break; // Receiver was dropped
                }

                position += chunk_size;
            }
        });

        Self {
            bytes_sent,
            inner: rx,
        }
    }
}

impl futures_util::Stream for ProgressTrackingStream {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;

        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(result)) => {
                // Update the bytes sent counter
                if let Ok(chunk) = &result {
                    let mut bytes_sent = self.bytes_sent.lock().unwrap();
                    *bytes_sent += chunk.len() as u64;
                }
                Poll::Ready(Some(result))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Progress callback function type
///
/// A boxed function that takes an optional percentage and bytes sent,
/// and returns a Result.
pub type ProgressCallback =
    Box<dyn Fn(Option<u8>, Option<u64>) -> Result<(), String> + Send + Sync>;

/// Upload configuration with retry settings
#[derive(Debug, Clone)]
pub struct UploadParams {
    /// Number of retry attempts
    pub retry_count: u32,
    /// Delay between retry attempts
    pub retry_spacing: std::time::Duration,
    /// Chunk size for streaming
    pub chunk_size: usize,
}

impl Default for UploadParams {
    fn default() -> Self {
        Self {
            retry_count: 3,
            retry_spacing: std::time::Duration::from_secs(2),
            chunk_size: 64 * 1024, // 64 KB
        }
    }
}

/// Uploads data to a NIP-96 server with progress callback
///
/// This function extends the standard NIP-96 upload_data function by adding progress reporting
/// via a callback function that is called periodically during the upload process.
///
/// # Arguments
///
/// * `signer` - The signer for NIP98 authorization
/// * `desc` - The server configuration
/// * `file_data` - The file data to upload
/// * `mime_type` - The MIME type of the file
/// * `proxy` - Optional proxy address
/// * `progress_callback` - The progress callback function
/// * `params` - Optional upload parameters with retry settings
/// * `config` - Optional upload client configuration
///
/// # Returns
///
/// A Result containing the URL of the uploaded file or an UploadError.
pub async fn upload_data_with_progress<T>(
    signer: &T,
    desc: &ServerConfig,
    file_data: Vec<u8>,
    mime_type: Option<&str>,
    proxy: Option<SocketAddr>,
    progress_callback: ProgressCallback,
    params: Option<UploadParams>,
    config: Option<UploadConfig>,
) -> Result<Url, UploadError>
where
    T: NostrSigner,
{
    let params = params.unwrap_or_default();
    let config = config.unwrap_or_default();

    let mut last_error = None;

    for attempt in 0..=params.retry_count {
        // Log retry attempt if not the first attempt
        if attempt > 0 {
            debug!("Retry attempt {} of {}", attempt, params.retry_count);
            // Sleep before retry
            tokio::time::sleep(params.retry_spacing).await;
        }

        match upload_attempt(
            signer,
            desc,
            file_data.clone(),
            mime_type,
            proxy,
            &progress_callback,
            &config,
            params.chunk_size,
        )
        .await
        {
            Ok(url) => return Ok(url),
            Err(e) => {
                last_error = Some(e);
                // Continue to next retry attempt
            }
        }
    }

    // All attempts failed, return the last error
    Err(last_error
        .unwrap_or_else(|| UploadError::UploadError("No upload attempts were made".to_string())))
}

/// Internal function that performs a single upload attempt
async fn upload_attempt<T>(
    signer: &T,
    desc: &ServerConfig,
    file_data: Vec<u8>,
    mime_type: Option<&str>,
    proxy: Option<SocketAddr>,
    progress_callback: &ProgressCallback,
    config: &UploadConfig,
    chunk_size: usize,
) -> Result<Url, UploadError>
where
    T: NostrSigner,
{
    // Build NIP98 Authorization header
    let payload: Sha256Hash = Sha256Hash::hash(&file_data);
    let data = HttpData::new(desc.api_url.clone(), HttpMethod::POST).payload(payload);
    let nip98_auth: String = data
        .to_authorization(signer)
        .await
        .map_err(|e| UploadError::UploadError(e.to_string()))?;

    // Create shared counter for tracking upload progress
    let bytes_sent = Arc::new(Mutex::new(0u64));
    let total_size = file_data.len() as u64;

    // Report initial progress (0%)
    progress_callback(Some(0), Some(0)).map_err(UploadError::UploadError)?;

    // Make client
    let client: Client = make_client(proxy, Some(config.clone()))?;

    // Create form with tracking stream
    let file_part = {
        let tracking_stream =
            ProgressTrackingStream::new(file_data.clone(), bytes_sent.clone(), chunk_size);
        let body = Body::wrap_stream(tracking_stream);
        let mut part = Part::stream(body).file_name("filename");

        // Set MIME type if provided
        if let Some(mime_str) = mime_type {
            part = part
                .mime_str(mime_str)
                .map_err(|_| UploadError::MultipartMimeError)?;
        }

        part
    };

    let form = multipart::Form::new().part("file", file_part);

    // Launch upload as a future, but don't await it yet
    let mut response_future = client
        .post(desc.api_url.clone())
        .header("Authorization", nip98_auth)
        .multipart(form)
        .send();

    // Create a future that polls the bytes_sent counter periodically
    let mut last_percentage = 0;
    let mut poll_interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    // Track stalled uploads
    let mut last_bytes_sent = 0u64;
    let mut stall_counter = 0;

    // Use tokio::select to concurrently wait for the response and report progress
    let response = loop {
        tokio::select! {
            // Check if the response is ready
            response = &mut response_future => {
                break response?;
            },
            // Report progress periodically
            _ = poll_interval.tick() => {
                let current_bytes = *bytes_sent.lock().unwrap();
                let percentage = if total_size > 0 {
                    ((current_bytes as f64 / total_size as f64) * 100.0) as u8
                } else {
                    0
                };

                // Check if upload is stalled
                if current_bytes == last_bytes_sent && percentage < 100 && percentage > 0 {
                    stall_counter += 1;
                    if stall_counter >= config.stall_threshold {
                        return Err(UploadError::UploadError("Upload stalled - no progress detected".to_string()));
                    }
                } else {
                    // Progress detected, reset stall counter
                    stall_counter = 0;
                    last_bytes_sent = current_bytes;
                }

                // Only report when percentage changes to reduce events
                if percentage > last_percentage {
                    if let Err(e) = progress_callback(Some(percentage), Some(current_bytes)) {
                        return Err(UploadError::UploadError(e));
                    }
                    last_percentage = percentage;
                }
            }
        }
    };

    // Report 100% completion
    progress_callback(Some(100), Some(total_size)).map_err(UploadError::UploadError)?;

    // Decode response
    let res: UploadResponse = response.json().await?;

    // Check status
    if res.status == UploadResponseStatus::Error {
        return Err(UploadError::UploadError(res.message));
    }

    // Extract url
    let nip94_event = res.nip94_event.ok_or(UploadError::ResponseDecodeError)?;
    match nip94_event.tags.find_standardized(TagKind::Url) {
        Some(TagStandard::Url(url)) => Ok(url.clone()),
        _ => Err(UploadError::ResponseDecodeError),
    }
}
