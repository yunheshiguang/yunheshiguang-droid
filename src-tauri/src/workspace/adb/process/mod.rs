mod failure;
mod output;
mod policy;
mod spawn;

use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use super::error::{AdbCommandError, AdbErrorKind};

pub use policy::CommandPolicy;
pub use spawn::configure_command;

use failure::{classify_exit_error, output_error_message};
use output::{StdoutTarget, finish_readers, receive_reader, spawn_reader};
use spawn::{terminate_child, wait_for_exit};

#[derive(Clone, Debug)]
pub struct ProcessOutput {
    pub stdout: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessExecutor;

impl ProcessExecutor {
    pub fn run(
        &self,
        program: &Path,
        args: &[OsString],
        policy: CommandPolicy,
    ) -> Result<ProcessOutput, AdbCommandError> {
        self.run_with_stdout(program, args, policy, StdoutTarget::Capture, None)
    }

    pub fn run_with_input(
        &self,
        program: &Path,
        args: &[OsString],
        policy: CommandPolicy,
        input: &[u8],
    ) -> Result<ProcessOutput, AdbCommandError> {
        self.run_with_stdout(program, args, policy, StdoutTarget::Capture, Some(input))
    }

    pub fn run_to_file(
        &self,
        program: &Path,
        args: &[OsString],
        policy: CommandPolicy,
        file: File,
    ) -> Result<(), AdbCommandError> {
        self.run_with_stdout(program, args, policy, StdoutTarget::File(file), None)
            .map(|_| ())
    }

    fn run_with_stdout(
        &self,
        program: &Path,
        args: &[OsString],
        policy: CommandPolicy,
        stdout_target: StdoutTarget,
        input: Option<&[u8]>,
    ) -> Result<ProcessOutput, AdbCommandError> {
        let mut command = Command::new(program);
        command
            .args(args)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure_command(&mut command);

        let mut child = command.spawn().map_err(AdbCommandError::from_io)?;
        if let (Some(input), Some(mut stdin)) = (input, child.stdin.take())
            && let Err(error) = stdin.write_all(input)
        {
            terminate_child(&mut child);
            return Err(AdbCommandError::from_io(error));
        }

        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            terminate_child(&mut child);

            return Err(AdbCommandError::new(
                AdbErrorKind::Io,
                "stdout or stderr pipe unavailable",
            ));
        };

        let limit = policy.max_output_bytes;
        let stdout_reader = spawn_reader(stdout, limit, stdout_target);
        let stderr_reader = spawn_reader(stderr, limit, StdoutTarget::Capture);
        let deadline = Instant::now() + policy.timeout;

        let status = match wait_for_exit(&mut child, policy.timeout) {
            Ok(status) => status,
            Err(error) => {
                finish_readers(stdout_reader, stderr_reader);

                return Err(error);
            }
        };

        let stdout = receive_reader(stdout_reader, deadline)?;
        let stderr = receive_reader(stderr_reader, deadline)?;

        if stdout.exceeded || stderr.exceeded {
            return Err(AdbCommandError::new(
                AdbErrorKind::OutputLimitExceeded,
                format!("command output exceeded the {limit} byte limit"),
            ));
        }

        if !status.success() {
            let message = output_error_message(&stderr.bytes, status.to_string());
            let kind = classify_exit_error(&message);

            return Err(if matches!(kind, AdbErrorKind::NonZeroExit) {
                AdbCommandError::non_zero(status.code(), message)
            } else {
                AdbCommandError::exited(kind, status.code(), message)
            });
        }

        Ok(ProcessOutput {
            stdout: stdout.bytes,
        })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn hands_input_to_the_program_instead_of_its_arguments() {
        let output = ProcessExecutor
            .run_with_input(
                Path::new("/bin/cat"),
                &[],
                CommandPolicy::new(Duration::from_secs(5), 1024),
                b"482913\n",
            )
            .unwrap();

        assert_eq!(output.stdout, b"482913\n");
    }
}
