pub mod formats;
pub mod metadata;
pub mod traits;
pub mod utils;
pub mod error;
pub mod mcp;
pub mod conversion;
pub mod progress;
pub mod image_optimizer;

pub use error::{EbookError, Result};
pub use traits::{EbookReader, EbookWriter, EbookOperator};
pub use metadata::Metadata;
pub use conversion::Converter;
pub use progress::{Progress, ProgressHandler, console_progress_callback, silent_progress_callback};
pub use formats::EpubVersion;
