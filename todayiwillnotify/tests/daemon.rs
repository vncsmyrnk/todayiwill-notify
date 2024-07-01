use std::{
    fs::{self, File},
    io::Write,
    thread,
    time::Duration,
};

use assert_cmd::Command;
use rustix::process::{self, Pid, Signal};
use todayiwill::{Appointment, AppointmentTime};

fn helper_write_to_appointment_data_file(content: &[u8]) {
    let data_file = dirs::data_dir().unwrap().join("todayiwill");
    fs::create_dir_all(data_file.parent().unwrap()).expect("Failed to create data dir");
    let mut file = File::create(data_file.to_str().unwrap()).expect("Failed to create test file");
    file.write_all(content)
        .expect("Failed to write to test file");
}

pub fn remove_all_daemon_files() {
    let app_data_dir = dirs::data_dir().unwrap().join("todayiwillnotify");
    if !app_data_dir.exists() {
        return;
    }
    for entry in app_data_dir.read_dir().expect("Failed to access data dir") {
        if let Ok(entry) = entry {
            match fs::remove_file(entry.path()) {
                Err(error) => panic!("Failed to remove data file. {error}"),
                _ => return,
            }
        }
    }
}

pub fn kill_daemon() {
    let file_result = fs::read_to_string(
        dirs::data_dir()
            .unwrap()
            .join("todayiwillnotify")
            .join("daemon.pid"),
    );
    let file_content = match file_result {
        Ok(content) => content,
        Err(..) => panic!("Impossible to read pid file, aborting"),
    };
    let pid = Pid::from_raw(file_content.trim().parse().unwrap())
        .expect("Failed to obtain daemon process");
    process::kill_process(pid, Signal::Kill).expect("Failed to kill daemon");
}

#[test]
fn daemon_should_log() {
    let appointment = Appointment::new(String::from("Wash the dishes"), AppointmentTime::now() + 2);
    helper_write_to_appointment_data_file(&format!("{}", appointment).into_bytes());
    remove_all_daemon_files();

    Command::cargo_bin("todayiwillnotify")
        .unwrap()
        .assert()
        .success();

    thread::sleep(Duration::from_secs(60 * 3));

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

    kill_daemon();
}
