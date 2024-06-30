use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher, Config, Event};
use daemonize::Daemonize;
use notify_rust::Notification;
use std::{fs::File, sync::mpsc::channel};
use std::thread;
use std::time::Duration;
use log::{info, error};
use env_logger::Env;

extern crate dirs;

fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let daemonize = Daemonize::new()
        .pid_file("/tmp/file_watcher_daemon.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .stdout(File::create("/tmp/file_watcher_daemon.out").unwrap())
        .stderr(File::create("/tmp/file_watcher_daemon.err").unwrap());

    match daemonize.start() {
        Ok(_) => info!("Daemonized successfully"),
        Err(e) => error!("Error daemonizing: {}", e),
    }

    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default().with_poll_interval(Duration::from_secs(10)))?;
    watcher.watch(&dirs::data_dir().unwrap().join("todayiwill").join("appointments_30062024.txt"), RecursiveMode::NonRecursive)?;

    info!("Watching for file changes...");

    let _handle = thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(event) => handle_event(event.unwrap()),
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

// Function to handle events.
fn handle_event(_event: Event) {
    Notification::new()
        .summary("Appointment change")
        .body("File change")
        .show()
        .expect("Failed to display message");
}
