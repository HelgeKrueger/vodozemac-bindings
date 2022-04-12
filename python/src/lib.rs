mod account;
mod group_sessions;
mod sas;
mod session;

use paste::paste;
use pyo3::{exceptions::PyValueError, prelude::*};
use thiserror::Error;

macro_rules! create_error {
    ($source:ty, $target:ident) => {
        paste! {
            pyo3::create_exception!(module, [<$target Exception>], pyo3::exceptions::PyException);
        }

        paste! {
            #[derive(Debug, Error)]
            #[error(transparent)]
            pub struct [<$target Error>] {
                source: $source,
            }
        }

        paste! {
            impl From<$source> for [<$target Error>] {
                fn from(e: $source) -> Self {
                    [<$target Error>] { source: e }
                }
            }
        }

        paste! {
            impl From<[<$target Error>]> for PyErr {
                fn from(e: [<$target Error>]) -> PyErr {
                    [<$target Exception>]::new_err(e.source.to_string())
                }
            }
        }
    };
}

create_error!(vodozemac::KeyError, Key);
create_error!(vodozemac::LibolmPickleError, LibolmPickle);
create_error!(vodozemac::megolm::SessionKeyDecodeError, SessionKeyDecode);

pyo3::create_exception!(module, PickleException, pyo3::exceptions::PyException);
pyo3::create_exception!(
    module,
    SessionCreationException,
    pyo3::exceptions::PyException
);
pyo3::create_exception!(module, DecodeException, pyo3::exceptions::PyException);
pyo3::create_exception!(module, SasException, pyo3::exceptions::PyException);
pyo3::create_exception!(
    module,
    OlmDecryptionException,
    pyo3::exceptions::PyException
);
pyo3::create_exception!(
    module,
    MegolmDecryptionException,
    pyo3::exceptions::PyException
);

#[derive(Debug, Error)]
pub enum MegolmDecryptionError {
    #[error(transparent)]
    Decode(#[from] vodozemac::DecodeError),
    #[error(transparent)]
    Decryption(#[from] vodozemac::megolm::DecryptionError),
}

impl From<MegolmDecryptionError> for PyErr {
    fn from(e: MegolmDecryptionError) -> Self {
        match e {
            MegolmDecryptionError::Decode(e) => DecodeException::new_err(e.to_string()),
            MegolmDecryptionError::Decryption(e) => {
                MegolmDecryptionException::new_err(e.to_string())
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum SasError {
    #[error(transparent)]
    Mac(#[from] vodozemac::Base64DecodeError),
    #[error(transparent)]
    Key(#[from] vodozemac::KeyError),
    #[error(transparent)]
    Sas(#[from] vodozemac::sas::SasError),
    #[error("The Sas object has already been used once.")]
    Used,
}

impl From<SasError> for PyErr {
    fn from(e: SasError) -> Self {
        match e {
            SasError::Key(e) => KeyException::new_err(e.to_string()),
            SasError::Sas(e) => SasException::new_err(e.to_string()),
            SasError::Mac(e) => SasException::new_err(e.to_string()),
            SasError::Used => SasException::new_err(e.to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error(transparent)]
    Key(#[from] vodozemac::KeyError),
    #[error(transparent)]
    Decode(#[from] vodozemac::DecodeError),
    #[error(transparent)]
    Decryption(#[from] vodozemac::olm::DecryptionError),
    #[error(transparent)]
    Creation(#[from] vodozemac::olm::SessionCreationError),
    #[error("Invalid message type, a pre-key message is needed to create a Session")]
    InvalidMessageType,
}

impl From<SessionError> for PyErr {
    fn from(e: SessionError) -> Self {
        match e {
            SessionError::Key(e) => KeyException::new_err(e.to_string()),
            SessionError::Decode(e) => DecodeException::new_err(e.to_string()),
            SessionError::Decryption(e) => OlmDecryptionException::new_err(e.to_string()),
            SessionError::Creation(e) => SessionCreationException::new_err(e.to_string()),
            SessionError::InvalidMessageType => PyValueError::new_err(e.to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum PickleError {
    #[error("The pickle key doesn't have the correct size, got {0}, expected 32 bytes")]
    InvalidKeySize(usize),
    #[error(transparent)]
    Unpickling(#[from] vodozemac::PickleError),
}

impl From<PickleError> for PyErr {
    fn from(e: PickleError) -> Self {
        PickleException::new_err(e.to_string())
    }
}

#[pyclass]
pub struct OlmMessage {
    #[pyo3(get)]
    ciphertext: String,
    #[pyo3(get)]
    message_type: usize,
}

#[pymethods]
impl OlmMessage {
    #[new]
    pub fn new(message_type: usize, ciphertext: &str) -> Self {
        Self {
            ciphertext: ciphertext.to_owned(),
            message_type,
        }
    }
}

#[pymodule]
#[pyo3(name = "vodozemac")]
fn mymodule(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<account::Account>()?;
    m.add_class::<session::Session>()?;
    m.add_class::<OlmMessage>()?;
    m.add_class::<sas::Sas>()?;
    m.add_class::<group_sessions::GroupSession>()?;
    m.add_class::<group_sessions::InboundGroupSession>()?;

    m.add("KeyException", py.get_type::<KeyException>())?;
    m.add("DecodeException", py.get_type::<DecodeException>())?;
    m.add("LibolmPickleException", py.get_type::<LibolmPickleException>())?;
    m.add("SessionKeyDecodeException", py.get_type::<SessionKeyDecodeException>())?;
    m.add("PickleException", py.get_type::<PickleException>())?;
    m.add("SessionCreationException", py.get_type::<SessionCreationException>())?;
    m.add("SasException", py.get_type::<SasException>())?;
    m.add("OlmDecryptionException", py.get_type::<OlmDecryptionException>())?;
    m.add("MegolmDecryptionException", py.get_type::<MegolmDecryptionException>())?;

    Ok(())
}
