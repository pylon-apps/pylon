//! Core functionality for the Pylon application.
//!
//! This library derives from and wraps over the [`magic-wormhole`] library to provide custom types and functionality.
//!
//! [`magic-wormhole`]: https://crates.io/crates/magic-wormhole

mod consts;

use std::borrow::Cow;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;

use futures::AsyncRead;
use magic_wormhole::rendezvous::DEFAULT_RENDEZVOUS_SERVER;
use magic_wormhole::transfer::{self, AppVersion, ReceiveRequest, TransferError};
use magic_wormhole::transit::{
    Abilities, RelayHint, RelayHintParseError, TransitInfo, DEFAULT_RELAY_SERVER,
};
use magic_wormhole::{AppConfig, AppID, Wormhole, WormholeError};
use thiserror::Error;

use consts::APP_ID;

/// Awaitable object that will perform the client-client handshake and yield the wormhole object on success.
type Handshake = dyn Future<Output = Result<Wormhole, WormholeError>>;

/// Custom error type for the various errors a Pylon may encounter.
///
/// These could be errors generated by the underlying wormhole library (some of which we handle explicitly and some of
/// which we don't), or custom validation/other errors that we may want to return.
#[derive(Debug, Error)]
pub enum PylonError {
    /// Wormhole code generation failed for some reason.
    /// Possibly because the underlying wormhole has already been initialized.
    #[error("Error generating wormhole code: {0}")]
    CodegenError(Box<str>),
    /// The provided relay server URL could not be parsed.
    /// This is just a wrapper over the underlying wormhole library's error of the same name.
    #[error("Error parsing relay server URL")]
    RelayHintParseError(
        #[from]
        #[source]
        RelayHintParseError,
    ),
    /// Error occured during the transfer.
    /// This is just a wrapper over the underlying womhole library's error of the same name.
    #[error("Error occured during transfer")]
    TransferError(
        #[from]
        #[source]
        TransferError,
    ),
    /// An error occured with the underlying wormhole library that we aren't explicitly matching against.
    #[error(transparent)]
    InternalError(#[from] WormholeError),
    /// Generic error messages.
    #[error("An error occured: {0}")]
    Error(Box<str>),
}

/// Configuration values for the Pylon.
pub struct PylonConfig {
    /// The ID of your application.
    pub id: String,
    /// The wormhole rendezvous server's URL.
    pub rendezvous_url: String,
    /// The wormhole relay server's URL.
    pub relay_url: String,
}

impl Default for PylonConfig {
    fn default() -> Self {
        Self {
            id: APP_ID.into(),
            rendezvous_url: DEFAULT_RENDEZVOUS_SERVER.into(),
            relay_url: DEFAULT_RELAY_SERVER.into(),
        }
    }
}

// TODO: improve documentation
/// High-level wrapper over a magic-wormhole that allows for secure file-transfers.
pub struct Pylon {
    handshake: Option<Box<Handshake>>,
    wormhole: Option<Wormhole>,
    transfer_request: Option<ReceiveRequest>,
    relay_url: String,
    config: AppConfig<AppVersion>,
}

impl Pylon {
    // TODO: add example(s)
    /// Creates a new Pylon using the specified config.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use. (Can use `Default::default()`).
    pub fn new(config: PylonConfig) -> Self {
        Self {
            handshake: None,
            wormhole: None,
            transfer_request: None,
            relay_url: config.relay_url,
            config: AppConfig {
                id: AppID(Cow::from(config.id)),
                rendezvous_url: Cow::from(config.rendezvous_url),
                app_version: AppVersion {},
            },
        }
    }

    // TODO: add example(s)
    /// Returns a generated wormhole code and connects to the rendezvous server.
    ///
    /// # Arguments
    ///
    /// * `code_length` - The required length of the wormhole code.
    pub async fn gen_code(&mut self, code_length: usize) -> Result<String, PylonError> {
        if let Some(_) = &self.handshake {
            return Err(PylonError::CodegenError(
                String::from("The current Pylon already has a pending handshake").into_boxed_str(),
            ));
        }

        if let Some(_) = &self.wormhole {
            return Err(PylonError::CodegenError(
                String::from("The current Pylon has already been initialized").into_boxed_str(),
            ));
        }

        let (welcome, handshake) =
            Wormhole::connect_without_code(self.config.clone(), code_length).await?;
        self.handshake = Some(Box::new(handshake));

        Ok(welcome.code.0)
    }

    // TODO: add example(s)
    /// Sends a file over the wormhole network to the receiver Pylon.
    ///
    /// # Arguments
    ///
    /// * `file` - The file reader of the file to send.
    /// * `file_name` - The name of the file.
    /// * `file_size` - The size of the file. **NOTE**: You must ensure this argument correctly matches the actual bytes
    ///                 contained in the file reader.
    /// * `progress_handler` - Callback function that accepts the number of bytes sent and the total number of bytes to send.
    /// * `cancel_handler` - Callback function to request cancellation of the file transfer.
    pub async fn send_file<F, N, P, C>(
        &mut self,
        file: &mut F,
        file_name: N,
        file_size: u64,
        progress_handler: P,
        cancel_handler: C,
    ) -> Result<(), PylonError>
    where
        F: AsyncRead + Unpin,
        N: Into<PathBuf>,
        P: FnMut(u64, u64) + 'static,
        C: Future<Output = ()>,
    {
        // TODO: allow caller to specify transit handler, abilities and relay hints
        let transit_handler = |_: TransitInfo, _: SocketAddr| {};
        let transit_abilities = Abilities::ALL_ABILITIES;
        // TODO: don't unwrap
        let relay_hints = vec![RelayHint::from_urls(
            None,
            [self.relay_url.parse().unwrap()],
        )?];

        let sender = match self.wormhole.take() {
            None => return Err(PylonError::Error("Wormhole not initialized".into())),
            Some(wh) => transfer::send_file(
                wh,
                relay_hints,
                file,
                file_name,
                file_size,
                transit_abilities,
                transit_handler,
                progress_handler,
                cancel_handler,
            ),
        };
        sender.await?;

        Ok(())
    }

    // TODO: add example(s)
    /// Initiates a request for a file transfer from the sender Pylon.
    ///
    /// # Arguments
    ///
    /// * `cancel_handler` - Callback function to request cancellation of the file transfer.
    pub async fn request_file<C: Future<Output = ()>>(
        &mut self,
        cancel_handler: C,
    ) -> Result<(), PylonError> {
        // TODO: allow caller to specify transit abilities and relay hints
        let transit_abilities = Abilities::ALL_ABILITIES;
        // TODO: don't unwrap
        let relay_hints = vec![RelayHint::from_urls(
            None,
            [self.relay_url.parse().unwrap()],
        )?];

        let request = match self.wormhole.take() {
            None => return Err(PylonError::Error("Wormhole not initialized".into())),
            Some(wh) => {
                transfer::request_file(wh, relay_hints, transit_abilities, cancel_handler).await?
            }
        };
        self.transfer_request = request;

        Ok(())
    }

    /// Destroys the Pylon.
    ///
    /// Currently, we just drop the Pylon. A cleaner shutdown process MAY be implemented in the future, but that depends
    /// on progress in the underlying [`magic-wormhole`] library's clean shutdown implementation.
    ///
    /// [`magic-wormhole`]: https://crates.io/crates/magic-wormhole
    pub fn destroy(self) {
        drop(self);
    }
}
