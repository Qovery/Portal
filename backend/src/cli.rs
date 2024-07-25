use std::path::PathBuf;

use clap::Parser;

/// Torii is a simple, powerful and extensible open-source Internal Developer Portal, just pass `-h`
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct CLI {
    /// Torii configuration file
    #[clap(short, long, value_name = "configuration file")]
    pub config: PathBuf,
}
