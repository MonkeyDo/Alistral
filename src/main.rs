use clap::Parser;
use color_eyre::eyre::Ok;

use database::cleanup::cleanup_database;
use database::get_conn;
use models::cli::Cli;

use crate::utils::println_cli;

pub mod api;
pub mod core;
pub mod database;
pub mod datastructures;
pub mod models;
/// This is the module containing all the different tools of this app
pub mod tools;
pub mod utils;

pub use crate::models::error::Error;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    //let mut clog = colog::default_builder();
    //clog.filter(None, log::LevelFilter::Trace);
    //clog.init();

    let cli = Cli::parse();

    let post_run = cli.run().await.expect("An error occured in the app");

    if post_run {
        println_cli("Optional cleanup - This is fine to cancel");
        println_cli("Cleaning some old entries...");
        cleanup_database(&mut *get_conn().await)
            .await
            .expect("Error while cleaning the database");
        println_cli("Done!");
    }

    println_cli("Have a nice day!");
    Ok(())
}
