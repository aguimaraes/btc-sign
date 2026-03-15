use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::process;

use clap::{Parser, Subcommand};

use btc_sign::display;
use btc_sign::output;
use btc_sign::psbt;
use btc_sign::sign;
use btc_sign::wif;

#[derive(Parser)]
#[command(name = "btc-sign")]
#[command(about = "Minimal offline Bitcoin transaction signer for cold storage")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a PSBT without signing (read-only, no key needed)
    Inspect {
        /// Path to PSBT file
        psbt_file: String,
    },

    /// Sign a PSBT with a WIF private key
    Sign {
        /// Path to PSBT file
        psbt_file: String,

        /// Output path for signed PSBT (use "-" for stdout as base64)
        #[arg(long)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { psbt_file } => {
            run_inspect(&psbt_file);
        }
        Commands::Sign { psbt_file, output } => {
            run_sign(&psbt_file, &output);
        }
    }
}

fn run_inspect(psbt_file: &str) {
    let psbt = match psbt::load(Path::new(psbt_file)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    let mut stderr = io::stderr().lock();
    if let Err(e) = display::display_psbt(&psbt, &mut stderr) {
        eprintln!("error: failed to display transaction: {}", e);
        process::exit(1);
    }
}

fn run_sign(psbt_file: &str, output_path: &str) {
    // Load PSBT.
    let mut psbt = match psbt::load(Path::new(psbt_file)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    // Display transaction details on stderr.
    let mut stderr = io::stderr().lock();
    if let Err(e) = display::display_psbt(&psbt, &mut stderr) {
        eprintln!("error: failed to display transaction: {}", e);
        process::exit(1);
    }
    drop(stderr);

    // Prompt for WIF.
    eprint!("\nEnter WIF private key: ");
    io::stderr().flush().ok();

    let wif_string = match read_wif() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to read key: {}", e);
            process::exit(1);
        }
    };

    // Decode WIF.
    let wif_key = match wif::decode_wif(&wif_string) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    // Drop the WIF string — we have the decoded key.
    drop(wif_string);

    // Verify key matches at least one input.
    let matching = match sign::count_matching_inputs(&psbt, &wif_key) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    if matching == 0 {
        eprintln!("error: private key does not match any input address");
        process::exit(1);
    }

    eprintln!("\nKey matches {} input(s).", matching);

    // Approval prompt.
    eprint!("Type 'approve' to sign this transaction: ");
    io::stderr().flush().ok();

    let mut approval = String::new();
    if let Err(e) = io::stdin().lock().read_line(&mut approval) {
        eprintln!("error: failed to read approval: {}", e);
        process::exit(1);
    }

    if approval.trim() != "approve" {
        eprintln!("signing aborted");
        process::exit(1);
    }

    // Sign.
    let signed = match sign::sign_psbt(&mut psbt, &wif_key) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    // wif_key is dropped here — ZeroizeOnDrop clears the bytes.
    drop(wif_key);

    // Write output.
    if let Err(e) = output::write_psbt(&psbt, output_path) {
        eprintln!("error: {}", e);
        process::exit(1);
    }

    eprintln!("signed {} input(s), wrote to {}", signed, output_path);
}

/// Read a line from stdin without echoing (on Unix terminals).
/// Falls back to normal reading if stdin is not a terminal.
fn read_wif() -> io::Result<String> {
    if io::stdin().is_terminal() {
        read_line_no_echo()
    } else {
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        Ok(line.trim_end().to_string())
    }
}

/// Read a line from stdin with echo disabled.
#[cfg(unix)]
fn read_line_no_echo() -> io::Result<String> {
    use std::os::unix::io::AsRawFd;

    let stdin_fd = io::stdin().as_raw_fd();

    // Get current terminal attributes.
    let mut termios: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(stdin_fd, &mut termios) } != 0 {
        return Err(io::Error::last_os_error());
    }

    let original = termios;

    // Disable echo.
    termios.c_lflag &= !libc::ECHO;
    if unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios) } != 0 {
        return Err(io::Error::last_os_error());
    }

    // Read the line.
    let mut line = String::new();
    let result = io::stdin().lock().read_line(&mut line);

    // Restore original terminal attributes.
    unsafe {
        libc::tcsetattr(stdin_fd, libc::TCSANOW, &original);
    }

    // Print newline since echo was off.
    eprintln!();

    result?;
    Ok(line.trim_end().to_string())
}

/// Fallback for non-Unix: read with echo.
#[cfg(not(unix))]
fn read_line_no_echo() -> io::Result<String> {
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim_end().to_string())
}
