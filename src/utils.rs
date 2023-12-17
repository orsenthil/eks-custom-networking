use std::{io, thread};
use std::process::{Command, Stdio};
use std::io::{BufRead, Read};

pub fn execute_command(command: &str) -> io::Result<()> {
    let command_parts: Vec<&str> = command.split_whitespace().collect();

    if let Some(first) = command_parts.get(0) {
        let args = &command_parts[1..];
        let mut command = Command::new(first);
        command.args(args);

        let mut child = command.stdout(Stdio::piped()).spawn()?;

        if let Some(stdout) = child.stdout.take() {
            let reader = io::BufReader::new(stdout);

            let _handle = thread::spawn(move || {
                reader.lines().map(|line| line.unwrap())
                    .for_each(|line| println!("{}", line));
            });

            child.wait()?;
        }
    }

    Ok(())
}

pub fn read_file_to_string(filename: &str) -> io::Result<String> {
    let mut file = std::fs::File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn execute_capture_output(command: &str) -> io::Result<String> {
    let command_parts: Vec<&str> = command.split_whitespace().collect();

    if let Some(first) = command_parts.get(0) {
        let args = &command_parts[1..];
        let mut command = Command::new(first);
        command.args(args);

        let output = command.output()?;

        if output.status.success() {
            return Ok(String::from_utf8(output.stdout).unwrap());
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "Error"));
        }
    }

    Ok(String::from(""))
}

/// Write to a local file and return file name.
pub fn write_to_file(filename: &str, contents: &str) -> io::Result<String> {
    std::fs::write(filename, contents)?;
    Ok(filename.to_string())
}
