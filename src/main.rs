use std::env;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{self, Duration};

async fn run_flash_command(
    id: &str,
    timeout: &str,
    log_file: &mut tokio::fs::File,
    binary_dir: &str,
    count: usize,
) -> bool {
    let command = "BootCommander";
    let id_arg = format!("-tid={}", id);
    let timeout_arg = format!("-t1={}", timeout);
    let dir_arg = format!("{}", binary_dir);
    let args = vec![
        "-t=xcp_can",
        "-d=can0",
        "-b=250000",
        &timeout_arg,
        &id_arg,
        "-xid=1",
        &dir_arg,
    ];
    println!(
        "Running command for ID {} with timeout {}, binary dir: {}",
        id, timeout, dir_arg
    );

    let mut child = Command::new(command)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout).lines();

    let mut output = String::new();
    let mut success = false;

    while let Ok(line) = time::timeout(Duration::from_secs(300), reader.next_line()).await {
        if let Some(line) = line.expect("Failed to read line") {
            println!("{}", line);
            output.push_str(&line);
            output.push('\n');

            if line.contains("Finishing programming session...[[32mOK[0m]") {
                success = true;
            }
        } else {
            break; // End of output stream
        }
    }

    if !success {
        log_file
            .write_all(format!("{} - FoC for {}: Not OK\n", count, id_arg).as_bytes())
            .await
            .expect("Failed to write to log file");
        log_file
            .write_all(b"Stdout:\n")
            .await
            .expect("Failed to write to log file");
        log_file
            .write_all(output.as_bytes())
            .await
            .expect("Failed to write to log file");
    } else {
        log_file
            .write_all(format!("{} - FoC for {}: OK\n", count, id_arg).as_bytes())
            .await
            .expect("Failed to write to log file");
    }

    child.kill().await.expect("Failed to kill process");

    success
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: {} <timeout_in_ms> <number_of_repeats> <118binary_directory> <148binary_directory>", args[0]);
        std::process::exit(1);
    }

    let timeout = &args[1];
    let num_repeats: usize = args[2].parse().expect("Invalid number of repeats");
    let dir_118 = &args[3];
    let dir_148 = &args[4];

    if (!Path::new(dir_118).exists() || !Path::new(dir_118).is_file())
        || (!Path::new(dir_148).exists() || !Path::new(dir_148).is_file())
    {
        eprintln!("Error: The provided binary directory path is not valid or does not exist.");
        std::process::exit(1);
    }

    let log_file = "error_log.txt";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .await
        .expect("Cannot open log file");

    for i in 0..num_repeats {
        println!("Iteration {}", i + 1);

        time::sleep(Duration::from_secs(5)).await;

        let res118 = run_flash_command("118", timeout, &mut file, dir_118, i + 1).await;
        if !res118 {
            println!("Flashing S32K118 failed on iteration {}", i + 1);
        }
        time::sleep(Duration::from_secs(5)).await;

        let res148 = run_flash_command("148", timeout, &mut file, dir_148, i + 1).await;
        if !res148 {
            println!("Flashing S32K148 failed on iteration {}", i + 1);
        }
    }
    println!("Completed {} iterations.", num_repeats);
}
