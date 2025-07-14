fn main() {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::process::{Command, Stdio};

    let output = Command::new("bash")
        .arg("test.sh")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let combined_output = String::from_utf8_lossy(&output.stdout);
    let mut debug_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("test.debug.log")?;
    let mut regular_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("test.log")?;

    for line in combined_output.lines() {
        // Write all lines to debug log
        writeln!(debug_log, "{}", line)?;

        // Check if this is a trace line
        if !is_xtrace_line(line) {
            writeln!(regular_log, "{}", line)?;
        }
    }

    fn is_xtrace_line(line: &str) -> bool {
        // Must start with '+'
        if !line.starts_with('+') {
            return false;
        }

        // Count consecutive '+' characters from the start
        let plus_count = line.chars().take_while(|&c| c == '+').count();

        // Check if '[TRACE]' appears immediately after the '+' characters
        if line.len() > plus_count {
            line[plus_count..].starts_with("[TRACE]")
        } else {
            false
        }
    }
}
