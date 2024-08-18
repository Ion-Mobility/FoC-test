use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{self, Duration};

pub struct Foc;

impl Foc {
    pub async fn run_flash_command(
        id: &str,
        timeout: &str,
        log_file: &mut tokio::fs::File,
        binary_dir: &str,
        count: u16,
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
        // let mut reader = BufReader::new(stdout).lines();
        let mut reader = BufReader::new(stdout);

        let mut output = String::new();
        let mut success = false;
        let mut finishing = false;

        while let Ok(_) = time::timeout(Duration::from_secs(300), reader.fill_buf()).await {
            let buffer = reader.buffer();

            if buffer.is_empty() {
                break; // End of output stream
            }
            let output_chunk = String::from_utf8_lossy(buffer).to_string();
            // Identify progress lines and handle them separately
            if output_chunk.contains("...") || output_chunk.contains('%') {
                // Print and update dynamic parts in place
                print!("\r{}", output_chunk.trim_end());
            } else {
                // For other lines, print normally
                println!("{}", output_chunk.trim_end());
            }
            output.push_str(&output_chunk);

            // clear the buffer to allow more data to read
            let len = buffer.len();
            reader.consume(len);
            if output_chunk.contains("Finishing programming session...") {
                finishing = true;
            }

            if finishing && output_chunk.contains("[[32mOK[0m]") {
                success = true;
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

    pub async fn setup() {
        //Some special for imx environment
        // Stop the hmi-service-manager
        let output = Command::new("systemctl")
            .arg("stop")
            .arg("hmi-service-manager")
            .output()
            .await
            .expect("Failed to stop hmi-service-manager");
        if !output.status.success() {
            eprintln!(
                "Failed to stop hmi-service-manager: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            println!("hmi-service-manager stopped successfully.");
        }

        // Execute imx-pwr-keep
        Command::new("sh")
            .arg("-c")
            .arg("imx-pwr-keep")
            .output()
            .await
            .expect("Failed to execute imx-pwr-keep");

        // Setup GPIO 30
        Command::new("sh")
            .arg("-c")
            .arg("echo 30 > /sys/class/gpio/export")
            .output()
            .await
            .expect("Failed to export GPIO 30");

        Command::new("sh")
            .arg("-c")
            .arg("echo out > /sys/class/gpio/gpio30/direction")
            .output()
            .await
            .expect("Failed to set GPIO 30 direction");

        Command::new("sh")
            .arg("-c")
            .arg("echo 1 > /sys/class/gpio/gpio30/value")
            .output()
            .await
            .expect("Failed to set GPIO 30 value");
    }
}
