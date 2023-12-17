use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::thread;

fn execute_command(command: &str) -> io::Result<()> {
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

fn execute_capture_output(command: &str) -> io::Result<String> {
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

fn main() {

    match execute_command("eksctl create cluster") {
        Ok(_) => (),
        Err(err) => println!("Error: {:?}", err),
    }

    match execute_capture_output("kubectl get nodes") {
        Ok(output) => println!("{}", output),
        Err(err) => println!("Error: {:?}", err),
    }

}