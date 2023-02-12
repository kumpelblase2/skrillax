use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    Register {
        username: String,
        password: String,
        passcode: Option<String>,
    },
    // Some other command ideas:
    // - Add news entry
    // - list news
    // - remove news
    // - ban user
    // - unban user
    // - set/unset gm
}
