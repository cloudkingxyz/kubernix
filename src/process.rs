use crate::Config;
use failure::{bail, format_err, Fallible};
use log::debug;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// The maximum wait time for processes to become ready
const READYNESS_TIMEOUT: u64 = 30;

/// A general process abstraction
pub struct Process {
    command: String,
    child: Child,
    log_file: PathBuf,
}

impl Process {
    /// Creates a new `Process` instance by spawning the provided command `cmd`.
    /// If the process creation fails, an `Error` will be returned.
    pub fn new(config: &Config, command: &[&str]) -> Fallible<Process> {
        // Prepare the commands
        let cmd = command
            .get(0)
            .map(|x| x.to_owned())
            .ok_or_else(|| format_err!("No valid command provided"))?;
        let args: Vec<&str> =
            command.iter().map(|x| x.to_owned()).skip(1).collect();

        let log_file = Path::new(&config.log.dir).join(format!("{}.log", cmd));
        let out_file = File::create(&log_file)?;
        let err_file = out_file.try_clone()?;

        // Spawn the process child
        let child = Command::new(cmd)
            .args(&args)
            .stderr(Stdio::from(err_file))
            .stdout(Stdio::from(out_file))
            .spawn()?;

        Ok(Process {
            command: cmd.to_owned() + &args.join(" "),
            child,
            log_file,
        })
    }

    // Wait for the process to become ready, by searching for the pattern in
    // every line of its output.
    pub fn wait_ready(&mut self, pattern: &str) -> Fallible<()> {
        debug!(
            "Waiting for process '{}' to become ready with pattern: '{}'",
            self.command, pattern
        );
        let now = Instant::now();

        while now.elapsed().as_secs() < READYNESS_TIMEOUT {
            let file = File::open(&self.log_file)?;
            let reader = BufReader::new(file);
            let line = reader
                .lines()
                .filter_map(|line| line.ok())
                .take_while(|_| now.elapsed().as_secs() < READYNESS_TIMEOUT)
                .find(|line| line.find(pattern).is_some())
                .ok_or_else(|| {
                    format_err!("Timed out waiting for process to become ready")
                });
            if let Ok(l) = line {
                debug!("Found pattern '{}' in line '{}'", pattern, l);
                return Ok(());
            }
            // Don't push too hard
            thread::sleep(Duration::from_secs(2));
        }

        bail!("Timed out waiting for process to become ready")
    }

    /// Stopping the process by killing it
    pub fn stop(&mut self) -> Fallible<()> {
        self.child.kill()?;
        Ok(())
    }
}