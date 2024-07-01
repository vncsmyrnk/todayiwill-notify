use daemonize::Daemonize;
use env_logger::Env;
use log::{error, info};
use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};
use notify_rust::Notification;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs, process, thread};
use std::{fs::File, sync::mpsc::channel};
use todayiwill::appointment::{self, Config as TiwConfig};
use todayiwill::AppointmentTime;

extern crate dirs;

fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let todayiwill_default_dir = dirs::data_dir().unwrap().join("todayiwill");

    let seconds_interval = env::var("SECONDS_INTERVAL")
        .unwrap_or_else(|_| "20".to_string())
        .parse::<u64>()
        .expect("SECONDS_INTERVAL must be a valid number");

    let seconds_to_notify = env::var("SECONDS_TO_NOTIFY")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<i32>()
        .expect("SECONDS_TO_NOTIFY must be a valid number");

    let todayiwill_data_path = PathBuf::from(
        env::var("TODAYIWILL_PATH")
            .unwrap_or_else(|_| todayiwill_default_dir.to_str().unwrap().to_string()),
    );

    let daemon_path = dirs::state_dir()
        .expect("Failed to obtain daemon dir")
        .join("todayiwillnotify");

    if !daemon_path.exists() {
        fs::create_dir(&daemon_path).expect("Failed to create daemon dir");
    }

    let daemonize = Daemonize::new()
        .pid_file(daemon_path.join("daemon.pid"))
        .chown_pid_file(true)
        .working_directory("/tmp")
        .stdout(File::create(daemon_path.join("daemon.out")).unwrap())
        .stdout(File::create(daemon_path.join("daemon.out")).unwrap())
        .stderr(File::create(daemon_path.join("daemon.err")).unwrap());

    match daemonize.start() {
        Ok(_) => info!("Daemonized successfully"),
        Err(e) => {
            error!("Error daemonizing: {}. Process exited", e);
            process::exit(1);
        }
    }

    info!("Setup > SLEEP_INTERVAL: {seconds_interval}; SECONDS_TO_NOTIFY: {seconds_to_notify}; TODAYIWILL_PATH: {}", todayiwill_data_path.to_str().unwrap());

    let appointments = Arc::new(Mutex::new(vec![]));
    let appointments_clone = appointments.clone();

    loop {
        if todayiwill_data_path.exists() {
            break;
        }
        error!("Data dir not found. Waiting 10 seconds for another retry");
        thread::sleep(Duration::from_secs(10));
    }

    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(
        tx,
        Config::default().with_poll_interval(Duration::from_secs(10)),
    )?;

    watcher.watch(&todayiwill_data_path, RecursiveMode::NonRecursive)?;

    info!(
        "Watching for file changes in \"{}\"",
        todayiwill_data_path.to_str().unwrap()
    );

    thread::spawn(move || loop {
        match rx.recv() {
            Ok(event) => {
                let mut appointments = appointments_clone.lock().unwrap();
                info!("Event: {:?}", event);
                let file_appointments = appointment::list::get_appointments_from_file(
                    &TiwConfig::default().appointment_file_path_current_day,
                );
                *appointments = file_appointments
                    .into_iter()
                    .filter(|appointment| appointment.time > AppointmentTime::now())
                    .collect();
                info!("Appointments updated")
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    });

    loop {
        let mut appointments = appointments.lock().unwrap();
        appointments.retain(|appointment| {
            if appointment.time > AppointmentTime::now()
                && AppointmentTime::now() + seconds_to_notify >= appointment.time
            {
                let notification_result = Notification::new()
                    .summary("Reminder")
                    .body(
                        format!(
                            "Your appointment \"{}\" begins at {}",
                            appointment.description, appointment.time
                        )
                        .as_str(),
                    )
                    .show();
                match notification_result {
                    Ok(..) => info!("Appointment \"{appointment}\" notified"),
                    Err(e) => error!("Failed to notify \"{appointment}\". Error: {e}"),
                }
                false
            } else {
                true
            }
        });
        info!("Appointments left to notify: {}", appointments.len());
        drop(appointments);
        thread::sleep(Duration::from_secs(seconds_interval));
    }
}
