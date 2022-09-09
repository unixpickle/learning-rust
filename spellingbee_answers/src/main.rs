use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
mod game;
use game::GameInfo;
use tokio;
use tokio::spawn;
use tokio::time::{sleep, sleep_until, Instant};

#[tokio::main]
async fn main() -> ExitCode {
    let game_state = Arc::new(Mutex::<Option<GameInfo>>::new(None));

    spawn(background_updater(game_state.clone()));

    // A webserver could be here and return the current game info.
    loop {
        println!("{:?}", game_state.lock().unwrap().deref());
        sleep(Duration::from_secs(10)).await;
    }

    // No need to return after infinite loop.
    // ExitCode::SUCCESS
}

async fn background_updater(game_state: Arc<Mutex<Option<GameInfo>>>) {
    loop {
        match GameInfo::fetch().await {
            Ok(info) => {
                // This looks contrived at first, and might seem simpler as
                //
                //     if let Some(x) = game_state.lock().unwrap().deref() {
                //          if x.expiration == info.expiration { ... }
                //     }
                //
                // But we cannot call sleep(...).await while game_state is locked
                // because a mutex guard intentionally does not implement Send.
                // Therefore, we must consume the mutex guard before we sleep, which
                // the current setup appears to do.
                if game_state
                    .lock()
                    .unwrap()
                    .deref()
                    .as_ref()
                    .map(|x| x.expiration == info.expiration)
                    == Some(true)
                {
                    eprintln!("Fetched identical game state. Retrying after 1 minute.");
                    sleep(Duration::from_secs(60)).await;
                } else {
                    println!("fetched new game info {:?}", info);
                    let deadline = info.expiration.clone();
                    *game_state.lock().unwrap().deref_mut() = Some(info);

                    match instant_for_system_time(deadline) {
                        Ok(deadline) => {
                            println!("sleeping until new game state is available...");
                            sleep_until(deadline).await;
                        }
                        Err(err) => {
                            eprintln!("error figuring out sleep time: {}", err.msg);
                            sleep(Duration::from_secs(60)).await;
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("error fetching game state: {}", err);
                sleep(Duration::from_secs(120)).await;
            }
        }
    }
}

struct TimeError {
    msg: String,
}

impl<D: Display> From<D> for TimeError {
    fn from(e: D) -> TimeError {
        TimeError {
            msg: format!("{}", e),
        }
    }
}

fn instant_for_system_time(t: SystemTime) -> Result<Instant, TimeError> {
    let duration = t.duration_since(SystemTime::now())?;
    let after = Instant::now()
        .checked_add(Duration::from_secs(1))
        .map_or(None, |x| x.checked_add(duration));
    if let Some(x) = after {
        Ok(x)
    } else {
        Err(TimeError {
            msg: String::from("could not add to current instant"),
        })
    }
}
