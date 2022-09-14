use clap::Parser;
use e_core_detection::{get_pe_partition_async, test_core};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// For current core
    #[clap(short, long)]
    single: bool,

    /// Get all cores
    #[clap(short, long)]
    all: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.single {
        unsafe { println!("{}", test_core()) }
        std::process::exit(0)
    }

    match get_pe_partition_async().await {
        Err(e) => {
            println!("{:#?}", e);
            std::process::exit(1)
        }
        Ok(res) => {
            println!("{}", res.formatted_string());
            std::process::exit(0)
        }
    }
}
