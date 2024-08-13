use std::env;
use std::fs::OpenOptions;
use tokio::io::{self, AsyncWriteExt};
use tokio::process::Command;

async fn run_flash_command(id: &str, timeout: &str, log_file: &mut tokio::fs::File) -> bool {
    let command = "BootCommander.ext";
    let id = format!("-tid={}", id);
    let timeout = format!("-t1={}", timeout);
    let args = vec![
        "-t=xcp_can",
        "-d=peak_pcanusb",
        "-b=250000",
        &timeout,
        &id,
        "-xid=1",
        "RealtimeECU.srec",
    ];
    println!("Running command for ID {} with timeout {}", id, timeout);

    let output = Command::new(command)
        .args(&args)
        .output()
        .await
        .expect("Failed to execute command");

    if !output.status.success() {
        log_file
            .write_all(format!("ID {}: Command failed\n", id).as_bytes())
            .await
            .expect("Failed to write to log file");
        log_file
            .write_all(
                format!("Stdout: {:?}\n", String::from_utf8_lossy(&output.stdout)).as_bytes(),
            )
            .await
            .expect("Failed to write to log file");
        log_file
            .write_all(
                format!("Stderr: {:?}\n", String::from_utf8_lossy(&output.stderr)).as_bytes(),
            )
            .await
            .expect("Failed to write to log file");
        false
    } else {
        println!("TID {}: Command succeeded", id);
        true
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let timeout = &args[1];
    let num_repeats: usize = args[2].parse().expect("Invalid number of repeats");

    let log_file = "error_log.txt";
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .await
        .expect("Cannot open log file");

    for i in 0..num_repeats {
        println!("Iteration {}", i + 1);

        let res118 = run_flash_command("118", timeout, &mut file).await;
        if !res118 {
            println!("Flashing S32K118 failed on iteration {}", i + 1);
        }

        let res148 = run_flash_command("148", timeout, &mut file).await;
        if !res148 {
            println!("Flashing S32K148 failed on iteration {}", i + 1);
        }
    }
    println!("Completed {} iterations.", num_repeats);
}
