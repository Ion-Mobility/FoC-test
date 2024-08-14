use std::env;
use std::{thread, time};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

async fn run_flash_command(
    id: &str,
    timeout: &str,
    log_file: &mut tokio::fs::File,
    binary_dir: &str,
    count: usize,
) -> bool {
    let command = "BootCommander";
    let id = format!("-tid={}", id);
    let timeout = format!("-t1={}", timeout);
    let dir = format!("{}", binary_dir);
    let args = vec![
        "-t=xcp_can",
        "-d=can0",
        "-b=250000",
        &timeout,
        &id,
        "-xid=1",
        &dir,
    ];
    println!(
        "Running command for ID {} with timeout {}, binary dir: {}",
        id, timeout, dir
    );

    let output = Command::new(command)
        .args(&args)
        .output()
        .await
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print stdout and stderr to console
    println!("{}", stdout);
    eprintln!("{}", stderr);

    if stdout.contains("Finishing programming session...[[32mOK[0m]") {
        log_file
            .write_all(format!("{} - FoC for {}: OK\n", count, id).as_bytes())
            .await
            .expect("Failed to write to log file");
        return true;
    } else {
        log_file
            .write_all(format!("{} - FoC for {}: Not OK\n", count, id).as_bytes())
            .await
            .expect("Failed to write to log file");
        log_file
            .write_all(b"Stdout:\n")
            .await
            .expect("Failed to write to log file");
        for line in stdout.lines() {
            log_file
                .write_all(line.as_bytes())
                .await
                .expect("Failed to write to log file");
            log_file
                .write_all(b"\n")
                .await
                .expect("Failed to write to log file");
        }
        return false;
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let timeout = &args[1];
    let num_repeats: usize = args[2].parse().expect("Invalid number of repeats");
    let dir = &args[3];

    let log_file = "error_log.txt";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .await
        .expect("Cannot open log file");

    for i in 0..num_repeats {
        println!("Iteration {}", i + 1);

        thread::sleep(time::Duration::from_secs(5));

        let res118 = run_flash_command("118", timeout, &mut file, dir, i + 1).await;
        if !res118 {
            println!("Flashing S32K118 failed on iteration {}", i + 1);
        }

        // let res148 = run_flash_command("148", timeout, &mut file).await;
        // if !res148 {
        //     println!("Flashing S32K148 failed on iteration {}", i + 1);
        // }
    }
    println!("Completed {} iterations.", num_repeats);
}
