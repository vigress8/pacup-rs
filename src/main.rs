use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Default)]
#[command(version, about)]
/// Help maintainers update pacscripts
struct Cli {
    /// Create a new branch and push the changes to git
    #[arg(short, long, default_value_t)]
    ship: bool,

    /// Package names or paths to pacscripts
    #[arg()]
    packages: Vec<PathBuf>,
}

fn main() {
    let opts = Cli::parse();
    println!("{:?}", opts);
}
