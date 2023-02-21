//! Core functionality for the Pylon application.
//!
//! This library derives from and wraps over the [`magic-wormhole`] library to provide custom types and functionality.
//!
//! [`magic-wormhole`]: https://crates.io/crates/magic-wormhole

mod consts;

use std::borrow::Cow;
use std::future::Future;

use magic_wormhole::rendezvous::DEFAULT_RENDEZVOUS_SERVER;
use magic_wormhole::transfer::AppVersion;
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
    /// An error occured with the underlying wormhole library that we aren't explicitly matching against.
    #[error("An internal error occured")]
    InternalError(
        #[from]
        #[source]
        WormholeError,
    ),
    /// Generic error messages.
    #[error("An error occured: {0}")]
    Error(Box<str>),
}

// TODO: improve documentation
/// High-level wrapper over a magic-wormhole that allows for secure file-transfers.
pub struct Pylon {
    handshake: Option<Box<Handshake>>,
    wormhole: Option<Wormhole>,
    config: AppConfig<AppVersion>,
}

impl Pylon {
    /// Creates a new Pylon with sane defaults.
    pub fn new() -> Self {
        Self {
            handshake: None,
            wormhole: None,
            config: AppConfig {
                id: AppID(Cow::from(APP_ID)),
                rendezvous_url: Cow::from(DEFAULT_RENDEZVOUS_SERVER),
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
