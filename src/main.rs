mod elmo_app;
mod elos;

use clap::Parser;
use elmo_app::ElmoApp;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "localhost:54321")]
    connection: String,
}

fn main() {
    let args = Args::parse();
    let _ = ratatui::run(|terminal| ElmoApp::run(&args.connection, terminal));
}
