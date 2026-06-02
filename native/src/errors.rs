use bitcode::{Decode, Encode};
use krill_common::QuicTransmissionError;

pub type RustFfiResult<T> = Result<T, RustFfiError>;

#[derive(Debug, PartialEq, Eq, uniffi::Error, thiserror::Error)]
pub enum RustFfiError {
    #[error("App storage already initialized")]
    AppStorageAlreadyInitialized,
    #[error("App storage not initialized")]
    AppStorageNotInitialized,
    #[error("Internal app error 0x000. Try restarting the app")]
    InternalAppError,
    #[error("{0}")]
    Storage(StorageError),
    #[error("Invalid Tai64N bytes")]
    Tai64NTimestampBytes,
    #[error("{0}")]
    Io(CustomErrorKind),
    #[error("{0}")]
    Quic(String),
    #[error("{0}")]
    Security(String),
    #[error("Unable to decode the bytes into `StoredOrgInfo`")]
    UnableToDecodeStoredOrgInfo,
}

#[uniffi::export]
impl RustFfiError {
    pub fn ui_message(&self) -> String {
        self.to_string()
    }
}

impl From<frost_dkg_types::FrostOpsError> for RustFfiError {
    fn from(error: frost_dkg_types::FrostOpsError) -> Self {
        Self::Security(error.to_string())
    }
}

impl From<QuicTransmissionError> for RustFfiError {
    fn from(error: QuicTransmissionError) -> Self {
        Self::Quic(error.to_string())
    }
}

impl From<tai64::Error> for RustFfiError {
    fn from(_: tai64::Error) -> Self {
        Self::Tai64NTimestampBytes
    }
}

#[derive(Debug, PartialEq, Eq, uniffi::Error, thiserror::Error, Clone, Encode, Decode)]
pub enum StorageError {
    #[error("Tried to insert a key that already exists")]
    KeyAlreadyExists,
    #[error("Tried to update a value but it's key was not found in the store")]
    KeyToUpdateNotFound,
    #[error("The Database is already open. Cannot acquire lock.")]
    DatabaseAlreadyOpen,
    #[error(
        "This savepoint is invalid or cannot be created. Savepoints become invalid when an older savepoint is restored after it was created, and savepoints cannot be created if the transaction is “dirty” (any tables have been opened)"
    )]
    InvalidSavepoint,
    #[error("`redb::RepairSession::abort` was called.")]
    RepairAborted,
    #[error("A persistent savepoint was modified")]
    PersistentSavepointModified,
    #[error("A persistent savepoint exists")]
    PersistentSavepointExists,
    #[error("An Ephemeral savepoint exists")]
    EphemeralSavepointExists,
    #[error("A transaction is still in-progress")]
    TransactionInProgress,
    #[error("The Database is corrupted. Error: `{0}`!")]
    Corrupted(String),
    #[error("The database file is in an old file format `{0}` and must be manually upgraded")]
    UpgradeRequired(u8),
    #[error("The value being inserted exceeds the maximum of 3GiB. Error: `{0}`!")]
    ValueTooLarge(String),
    #[error("Table types didn't match. Table: `{table}` - key: `{key}` - value: `{value}`!")]
    TableTypeMismatch {
        table: String,
        key: String,
        value: String,
    },
    #[error("The table is a multimap table. Error: `{0}`!")]
    TableIsMultimap(String),
    #[error("The table is not a multimap table. Error: `{0}`!")]
    TableIsNotMultimap(String),
    #[error("name: `{name}` - alignment: `{alignment}` - width: `{width:?}`")]
    TypeDefinitionChanged {
        name: String,
        alignment: String,
        width: Option<String>,
    },
    #[error("Table name does not match any table in database. Error: `{0}`!")]
    TableDoesNotExist(String),
    #[error("Table name already exists in the database. Error: `{0}`!")]
    TableExists(String),
    #[error("`{0}` - `{1}`")]
    TableAlreadyOpen(String, String),
    #[error("I/O error: `{}`", format!("{:?}", .0))]
    Io(CustomErrorKind),
    #[error("Database is already closed")]
    DatabaseClosed,
    #[error("A previous IO error occurred. The database must be closed and re-opened")]
    PreviousIo,
    #[error("Lock poisoned. Error: `{0}`!")]
    LockPoisoned(String),
    #[error("The transaction is still referenced by a table or other object")]
    ReadTransactionStillInUse,
    #[error("`non_exhaustive` reached!`")]
    Fatal,
}

impl From<redb::Error> for RustFfiError {
    fn from(value: redb::Error) -> Self {
        Self::Storage(value.into())
    }
}

impl From<redb::TableError> for RustFfiError {
    fn from(value: redb::TableError) -> Self {
        let value: redb::Error = value.into();
        Self::Storage(value.into())
    }
}

impl From<redb::TransactionError> for RustFfiError {
    fn from(value: redb::TransactionError) -> Self {
        let value: redb::Error = value.into();
        Self::Storage(value.into())
    }
}

impl From<redb::StorageError> for RustFfiError {
    fn from(value: redb::StorageError) -> Self {
        let value: redb::Error = value.into();
        Self::Storage(value.into())
    }
}

impl From<redb::CommitError> for RustFfiError {
    fn from(value: redb::CommitError) -> Self {
        let value: redb::Error = value.into();
        Self::Storage(value.into())
    }
}

impl From<redb::Error> for StorageError {
    fn from(value: redb::Error) -> Self {
        match value {
            redb::Error::DatabaseAlreadyOpen => Self::DatabaseAlreadyOpen,
            redb::Error::InvalidSavepoint => Self::InvalidSavepoint,
            redb::Error::RepairAborted => Self::RepairAborted,
            redb::Error::PersistentSavepointModified => Self::PersistentSavepointModified,
            redb::Error::PersistentSavepointExists => Self::PersistentSavepointExists,
            redb::Error::EphemeralSavepointExists => Self::EphemeralSavepointExists,
            redb::Error::TransactionInProgress => Self::TransactionInProgress,
            redb::Error::Corrupted(value) => Self::Corrupted(value),
            redb::Error::UpgradeRequired(value) => Self::UpgradeRequired(value),
            redb::Error::ValueTooLarge(value) => Self::ValueTooLarge(value.to_string()),
            redb::Error::TableTypeMismatch { table, key, value } => Self::TableTypeMismatch {
                table,
                key: key.name().to_string(),
                value: value.name().to_string(),
            },
            redb::Error::TableIsMultimap(value) => Self::TableIsMultimap(value),
            redb::Error::TableIsNotMultimap(value) => Self::TableIsNotMultimap(value),
            redb::Error::TypeDefinitionChanged {
                name,
                alignment,
                width,
            } => Self::TypeDefinitionChanged {
                name: name.name().to_string(),
                alignment: alignment.to_string(),
                width: width.map(|value| value.to_string()),
            },
            redb::Error::TableDoesNotExist(value) => Self::TableDoesNotExist(value),
            redb::Error::TableExists(value) => Self::TableExists(value),
            redb::Error::TableAlreadyOpen(value1, value2) => {
                Self::TableAlreadyOpen(value1, value2.to_string())
            }
            redb::Error::Io(value) => {
                let value: CustomErrorKind = value.kind().into();

                Self::Io(value)
            }
            redb::Error::DatabaseClosed => Self::DatabaseClosed,
            redb::Error::PreviousIo => Self::PreviousIo,
            redb::Error::LockPoisoned(value) => Self::LockPoisoned(value.to_string()),
            redb::Error::ReadTransactionStillInUse(_) => Self::ReadTransactionStillInUse,
            _ => Self::Fatal,
        }
    }
}

#[derive(
    Debug,
    Eq,
    uniffi::Error,
    thiserror::Error,
    Default,
    PartialEq,
    PartialOrd,
    Clone,
    Copy,
    Hash,
    Encode,
    Decode,
)]
pub enum CustomErrorKind {
    #[error("An entity was not found, often a file.")]
    /// An entity was not found, often a file.
    NotFound,
    #[error("The operation lacked the necessary privileges to complete.")]
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    #[error("The connection was refused by the remote server.")]
    /// The connection was refused by the remote server.
    ConnectionRefused,
    #[error("The connection was reset by the remote server.")]
    /// The connection was reset by the remote server.
    ConnectionReset,
    #[error("The remote host is not reachable.")]
    /// The remote host is not reachable.
    HostUnreachable,
    #[error("The network containing the remote host is not reachable.")]
    /// The network containing the remote host is not reachable.
    NetworkUnreachable,
    #[error("The connection was aborted (terminated) by the remote server.")]
    /// The connection was aborted (terminated) by the remote server.
    ConnectionAborted,
    #[error("The network operation failed because it was not connected yet.")]
    /// The network operation failed because it was not connected yet.
    NotConnected,
    #[error(
        "A socket address could not be bound because the address is already in use elsewhere."
    )]
    /// A socket address could not be bound because the address is already in use elsewhere.
    AddrInUse,
    #[error("A nonexistent interface was requested or the requested address was not local.")]
    /// A nonexistent interface was requested or the requested address was not local.
    AddrNotAvailable,
    #[error("The system's networking is down.")]
    /// The system's networking is down.
    NetworkDown,
    #[error("The operation failed because a pipe was closed.")]
    /// The operation failed because a pipe was closed.
    BrokenPipe,
    #[error("An entity already exists, often a file.")]
    /// An entity already exists, often a file.
    AlreadyExists,
    #[error(
     "The operation needs to block to complete, but the blocking operation was requested to not occur.")]
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock,
    #[error("A filesystem object is, unexpectedly, not a directory.")]
    /// A filesystem object is, unexpectedly, not a directory.
    ///
    /// For example, a filesystem path was specified where one of the intermediate directory
    /// components was, in fact, a plain file.
    NotADirectory,
    #[error("The filesystem object is, unexpectedly, a directory.")]
    /// The filesystem object is, unexpectedly, a directory.
    ///
    /// A directory was specified when a non-directory was expected.
    IsADirectory,
    #[error("A non-empty directory was specified where an empty directory was expected.")]
    /// A non-empty directory was specified where an empty directory was expected.
    DirectoryNotEmpty,
    #[error("The filesystem or storage medium is read-only, but a write operation was attempted.")]
    /// The filesystem or storage medium is read-only, but a write operation was attempted.
    ReadOnlyFilesystem,
    #[error("Stale network file handle.")]
    /// Stale network file handle.
    ///
    /// With some network filesystems, notably NFS, an open file (or directory) can be invalidated
    /// by problems with the network or server.
    StaleNetworkFileHandle,
    #[error("A parameter was incorrect.")]
    /// A parameter was incorrect.
    InvalidInput,
    #[error("Data not valid for the operation were encountered.")]
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: ErrorKind::InvalidInput
    InvalidData,
    #[error("The I/O operation's timeout expired, causing it to be canceled.")]
    /// The I/O operation's timeout expired, causing it to be canceled.
    TimedOut,
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: crate::io::Write::write
    /// [`Ok(0)`]: Ok
    #[error("An error returned when an operation could not be completed because a call to [`write`] returned [`Ok(0)`].")]
    WriteZero,
    #[error("The underlying storage (typically, a filesystem) is full.")]
    /// The underlying storage (typically, a filesystem) is full.
    ///
    /// This does not include out of quota errors.
    StorageFull,
    #[error("Seek on unseekable file.")]
    /// Seek on unseekable file.
    ///
    /// Seeking was attempted on an open file handle which is not suitable for seeking - for
    /// example, on Unix, a named pipe opened with `File::open`.
    NotSeekable,
    #[error("Filesystem quota or some other kind of quota was exceeded.")]
    /// Filesystem quota or some other kind of quota was exceeded.
    QuotaExceeded,
    #[error("File larger than allowed or supported.")]
    /// File larger than allowed or supported.
    ///
    /// This might arise from a hard limit of the underlying filesystem or file access API, or from
    /// an administratively imposed resource limitation.  Simple disk full, and out of quota, have
    /// their own errors.
    FileTooLarge,
    #[error("Resource is busy.")]
    /// Resource is busy.
    ResourceBusy,
    #[error("Executable file is busy.")]
    /// Executable file is busy.
    ///
    /// An attempt was made to write to a file which is also in use as a running program.  (Not all
    /// operating systems detect this situation.)
    ExecutableFileBusy,
    #[error("Deadlock (avoided).")]
    /// Deadlock (avoided).
    ///
    /// A file locking operation would result in deadlock.  This situation is typically detected, if
    /// at all, on a best-effort basis.
    Deadlock,
    #[error("Cross-device or cross-filesystem (hard) link or rename.")]
    /// Cross-device or cross-filesystem (hard) link or rename.
    CrossesDevices,
    #[error("Too many (hard) links to the same filesystem object.")]
    /// Too many (hard) links to the same filesystem object.
    ///
    /// The filesystem does not support making so many hardlinks to the same file.
    TooManyLinks,
    #[error("A filename was invalid.")]
    /// A filename was invalid.
    ///
    /// This error can also occur if a length limit for a name was exceeded.
    InvalidFilename,
    #[error("Program argument list too long.")]
    /// Program argument list too long.
    ///
    /// When trying to run an external program, a system or process limit on the size of the
    /// arguments would have been exceeded.
    ArgumentListTooLong,
    #[error("This operation was interrupted.")]
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    Interrupted,

    #[error("This operation is unsupported on this platform.")]
    /// This operation is unsupported on this platform.
    ///
    /// This means that the operation can never succeed.
    Unsupported,

    // ErrorKinds which are primarily categorisations for OS error
    // codes should be added above.
    //
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    #[error(
        "ErrorKinds which are primarily categorisations for OS error codes should be added above."
    )]
    UnexpectedEof,
    /// An operation could not be completed, because it failed
    /// to allocate enough memory.
    #[error(" An operation could not be completed, because it failed to allocate enough memory.")]
    OutOfMemory,

    // "Unusual" error kinds which do not correspond simply to (sets
    // of) OS error codes, should be added just above this comment.
    // `Other` and `Uncategorized` should remain at the end:
    //
    /// A custom error that does not fall under any other I/O error kind.
    ///
    /// This can be used to construct your own [`Error`]s that do not match any
    /// [`ErrorKind`].
    ///
    /// This [`ErrorKind`] is not used by the standard library.
    ///
    /// Errors from the standard library that do not fall under any of the I/O
    /// error kinds cannot be `match`ed on, and will only match a wildcard (`_`) pattern.
    /// New [`ErrorKind`]s might be added in the future for some of those.
    #[error("`Unusual` error kinds which do not correspond simply to (sets of) OS error codes, should be added just above this comment.")]
    Other,
    #[error("`InExhaustiveReached` reached, maybe std::io::ErrorKind has new variants")]
    /// `InExhaustiveReached` reached, maybe std::io::ErrorKind has new variants
    #[default]
    InExhaustiveReached,
}

impl From<std::io::ErrorKind> for CustomErrorKind {
    fn from(value: std::io::ErrorKind) -> Self {
        use std::io::ErrorKind as StdVariant;

        match value {
            StdVariant::NotFound => Self::NotFound,
            StdVariant::PermissionDenied => Self::PermissionDenied,
            StdVariant::ConnectionRefused => Self::ConnectionRefused,
            StdVariant::ConnectionReset => Self::ConnectionReset,
            StdVariant::HostUnreachable => Self::HostUnreachable,
            StdVariant::NetworkUnreachable => Self::NetworkUnreachable,
            StdVariant::ConnectionAborted => Self::ConnectionAborted,
            StdVariant::NotConnected => Self::NotConnected,
            StdVariant::AddrInUse => Self::AddrInUse,
            StdVariant::AddrNotAvailable => Self::AddrNotAvailable,
            StdVariant::NetworkDown => Self::NetworkDown,
            StdVariant::BrokenPipe => Self::BrokenPipe,
            StdVariant::AlreadyExists => Self::AlreadyExists,
            StdVariant::WouldBlock => Self::WouldBlock,
            StdVariant::NotADirectory => Self::NotADirectory,
            StdVariant::IsADirectory => Self::IsADirectory,
            StdVariant::DirectoryNotEmpty => Self::DirectoryNotEmpty,
            StdVariant::ReadOnlyFilesystem => Self::ReadOnlyFilesystem,
            StdVariant::StaleNetworkFileHandle => Self::StaleNetworkFileHandle,
            StdVariant::InvalidInput => Self::InvalidInput,
            StdVariant::InvalidData => Self::InvalidData,
            StdVariant::TimedOut => Self::TimedOut,
            StdVariant::WriteZero => Self::WriteZero,
            StdVariant::StorageFull => Self::StorageFull,
            StdVariant::NotSeekable => Self::NotSeekable,
            StdVariant::QuotaExceeded => Self::QuotaExceeded,
            StdVariant::FileTooLarge => Self::FileTooLarge,
            StdVariant::ResourceBusy => Self::ResourceBusy,
            StdVariant::ExecutableFileBusy => Self::ExecutableFileBusy,
            StdVariant::Deadlock => Self::Deadlock,
            StdVariant::CrossesDevices => Self::CrossesDevices,
            StdVariant::TooManyLinks => Self::TooManyLinks,
            StdVariant::InvalidFilename => Self::InvalidFilename,
            StdVariant::ArgumentListTooLong => Self::ArgumentListTooLong,
            StdVariant::Interrupted => Self::Interrupted,
            StdVariant::Unsupported => Self::Unsupported,
            StdVariant::UnexpectedEof => Self::UnexpectedEof,
            StdVariant::OutOfMemory => Self::OutOfMemory,
            StdVariant::Other => Self::Other,
            _ => Self::InExhaustiveReached,
        }
    }
}

impl From<std::io::ErrorKind> for RustFfiError {
    fn from(error: std::io::ErrorKind) -> Self {
        let value: CustomErrorKind = error.into();

        Self::Io(value)
    }
}

impl From<std::io::Error> for RustFfiError {
    fn from(error: std::io::Error) -> Self {
        let value: CustomErrorKind = error.kind().into();

        Self::Io(value)
    }
}
