use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about= None)]
///A command to test Flashing Over CAN from imx(host)
pub struct Args {
    /// Number of repeating Foc for 118&148
    #[arg(short, long, default_value_t = 1)]
    pub iter_num: u16,
    /// Path to 118 binary file
    #[arg(short, long)]
    pub rt_bin_path: String,
    ///Path to 148 binary file
    #[arg(short, long)]
    pub tl_bin_path: String,
}
