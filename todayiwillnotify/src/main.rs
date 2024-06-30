use daemonize::Daemonize;
use env_logger::Env;
use log::{error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use notify_rust::Notification;
use std::thread;
use std::time::Duration;
use std::{fs::File, sync::mpsc::channel};
use todayiwill::appointment::{self, Config as TiwConfig};

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
    let mut watcher: RecommendedWatcher = Watcher::new(
        tx,
        Config::default().with_poll_interval(Duration::from_secs(10)),
    )?;
    watcher.watch(
        &dirs::data_dir().unwrap().join("todayiwill"),
        RecursiveMode::NonRecursive,
    )?;

    info!("Watching for file changes...");

    thread::spawn(move || loop {
        match rx.recv() {
            Ok(event) => handle_event(event.unwrap()),
            Err(e) => error!("watch error: {:?}", e),
        }
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

// Function to handle events.
fn handle_event(event: Event) {
    info!("Event: {:?}", event);
    let appointments = appointment::list::get_appointments_from_file(
        &TiwConfig::default().appointment_file_path_current_day,
    );
    for appointment in appointments {
        info!("Appointment found: {:?}", appointment);
    }
    Notification::new()
        .summary("Appointment change")
        .body("File change")
        .show()
        .expect("Failed to display message");
}
