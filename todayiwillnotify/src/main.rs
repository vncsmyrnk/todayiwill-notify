use daemonize::Daemonize;
use env_logger::Env;
use log::{error, info};
use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};
use notify_rust::Notification;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs::File, sync::mpsc::channel};
use std::{process, thread};
use todayiwill::appointment::{self, Config as TiwConfig};
use todayiwill::AppointmentTime;

extern crate dirs;

fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let daemonize = Daemonize::new()
        .pid_file("/tmp/todayiwillnotify-daemon.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .stdout(File::create("/tmp/todayiwillnotify-daemon.out").unwrap())
        .stderr(File::create("/tmp/todayiwillnotify-daemon.err").unwrap());

    match daemonize.start() {
        Ok(_) => info!("Daemonized successfully"),
        Err(e) => {
            error!("Error daemonizing: {}. Process exited", e);
            process::exit(1);
        }
    }

    let appointments = Arc::new(Mutex::new(vec![]));
    let appointments_clone = appointments.clone();

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

    thread::spawn(move || loop {
        let mut appointments = appointments.lock().unwrap();
        appointments.retain(|appointment| {
            if appointment.time > AppointmentTime::now()
                && AppointmentTime::now() + 1 >= appointment.time
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
        thread::sleep(Duration::from_secs(20));
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
