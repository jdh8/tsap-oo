//! A desktop client for tsap-oo: the web page in a native window.
//!
//! The page is `web/`'s, unchanged, embedded at build time.  What differs is
//! where the engine runs and who sits at the table: `web/app.js` finds
//! `window.__TAURI__` and calls the commands below instead of loading the
//! wasm, and the three opponents search ([`WebGame::searching`]), which the
//! browser's do not.  Every command returns the same JSON string its
//! `WebGame` namesake hands the browser.

// No console window beside a release build on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{LazyLock, mpsc};
use tsap_oo_web::WebGame;

type Job = Box<dyn FnOnce(&mut WebGame) + Send>;

/// The engine thread's inbox.  One thread owns the game, so every call is a
/// job sent here, in order, and the window never waits on a search.
static ENGINE: LazyLock<mpsc::Sender<Job>> = LazyLock::new(|| {
    let (engine, inbox) = mpsc::channel::<Job>();
    std::thread::spawn(move || {
        // The page deals its own game first thing; this one is never shown.
        let mut game = WebGame::searching("0", "{}");
        for job in inbox {
            job(&mut game);
        }
    });
    engine
});

/// Run `job` on the engine thread and wait for its answer
async fn ask<R: Send + 'static>(job: impl FnOnce(&mut WebGame) -> R + Send + 'static) -> R {
    let (reply, mut answer) = tauri::async_runtime::channel(1);
    ENGINE
        .send(Box::new(move |game| {
            // The asker may have gone away (a closed window); nothing to do
            let _ = reply.try_send(job(game));
        }))
        .expect("the engine thread outlives the window");
    answer.recv().await.expect("the engine thread answers")
}

/// Deal a new game; `seed` and `settings` are [`WebGame::new`]'s
#[tauri::command]
async fn new_game(seed: String, settings: String) -> String {
    ask(move |game| {
        *game = WebGame::searching(&seed, &settings);
        game.snapshot()
    })
    .await
}

#[tauri::command]
async fn step() -> String {
    ask(WebGame::step).await
}

#[tauri::command]
async fn pass() -> String {
    ask(WebGame::pass).await
}

#[tauri::command]
async fn pong() -> String {
    ask(WebGame::pong).await
}

#[tauri::command]
async fn eat(index: usize) -> String {
    ask(move |game| game.eat(index)).await
}

#[tauri::command]
async fn discard(code: String) -> String {
    ask(move |game| game.discard(&code)).await
}

#[tauri::command]
async fn hint() -> String {
    ask(|game| game.hint()).await
}

#[tauri::command]
async fn next_round() -> String {
    ask(WebGame::next_round).await
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            new_game, step, pass, pong, eat, discard, hint, next_round,
        ])
        .run(tauri::generate_context!())
        .expect("the window opens");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::async_runtime::block_on;

    /// The commands share the one game: deal, then follow the hint through
    /// a whole round against the searching opponents.
    #[test]
    fn a_round_plays_out_over_the_commands() {
        block_on(async {
            let mut snapshot = new_game("7".into(), r#"{"threshold":8}"#.into()).await;
            assert!(snapshot.contains(r#""threshold":8"#));
            while !snapshot.contains(r#""phase":"finished""#) {
                let hints = hint().await;
                // The pick comes first: `[{"code":"RK",...` or `[]`.
                snapshot = match hints.split('"').nth(3) {
                    None => step().await,
                    Some("pass") => pass().await,
                    Some("pong") => pong().await,
                    Some(code) => match code.strip_prefix("eat") {
                        Some(index) => eat(index.parse().unwrap()).await,
                        None => discard(code.into()).await,
                    },
                };
            }
            assert!(next_round().await.contains(r#""phase":"#));
        });
    }
}
