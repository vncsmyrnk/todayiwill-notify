use std::{
    fs::{self, File},
    io::Write, thread, time::Duration,
};

use assert_cmd::Command;
use todayiwill::{Appointment, AppointmentTime};

fn helper_write_to_appointment_data_file(content: &[u8]) {
    let data_file = dirs::data_dir().unwrap().join("todayiwill");
    fs::create_dir_all(data_file.parent().unwrap()).expect("Failed to create data dir");
    let mut file = File::create(data_file.to_str().unwrap()).expect("Failed to create test file");
    file.write_all(content)
        .expect("Failed to write to test file");
}

#[test]
fn daemon_should_log() {
    // Not working. Maybe create a cli with commands "up" and "down" for running and stopping the
    // daemon
    let appointment = Appointment::new(String::from("Wash the dishes"), AppointmentTime::now() + 2);
    helper_write_to_appointment_data_file(&format!("{}", appointment).into_bytes());

    Command::cargo_bin("todayiwillnotify")
        .unwrap()
        .assert()
        .success();

    thread::sleep(Duration::from_secs(60*3));

    let daemon_pid_file = dirs::data_dir()
        .unwrap()
        .join("todayiwillnotify")
        .join(format!("daemon.pid"));

    let daemon_stdout_file = dirs::data_dir()
        .unwrap()
        .join("todayiwillnotify")
        .join(format!("daemon.pid"));

    let daemon_stderr_file = dirs::data_dir()
        .unwrap()
        .join("todayiwillnotify")
        .join(format!("daemon.pid"));

    assert!(daemon_pid_file.exists());
    assert!(daemon_stdout_file.exists());
    assert!(daemon_stderr_file.exists());
}
