use std::process::ExitCode;
mod game;
use game::GameInfo;
use tokio;

#[tokio::main]
async fn main() -> ExitCode {
    match GameInfo::fetch().await {
        Ok(info) => {
            println!("{:?}", info);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
    ExitCode::SUCCESS
}
