use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    pub path: Option<String>,
}

