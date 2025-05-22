use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::thread;
use std::time::SystemTime;


fn main() {
    // Open log file with timestamp
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let log_path = format!("/tmp/sysy-lsp-{}.log", timestamp);
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file");
    
    writeln!(log_file, "Server started at timestamp: {}", timestamp)
        .expect("Failed to write to log file");
    
    // Log server startup
    writeln!(log_file, "SySy Language Server starting up...")
        .expect("Failed to write to log file");
    
    // Spawn a thread to handle stdin/stdout communication
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    
    // Read the standard Language Server Protocol headers
    let mut headers = String::new();
    let mut content_length = 0;
    
    writeln!(log_file, "Waiting for LSP messages...")
        .expect("Failed to write to log file");
    
    loop {
        headers.clear();
        // Read headers
        loop {
            let mut header_line = String::new();
            if stdin_lock.read_line(&mut header_line).unwrap() == 0 {
                writeln!(log_file, "Input stream closed, exiting")
                    .expect("Failed to write to log file");
                return;
            }
            
            writeln!(log_file, "Header: {}", header_line.trim())
                .expect("Failed to write to log file");
            
            if header_line.trim().is_empty() {
                break;
            }
            
            headers.push_str(&header_line);
            
            if header_line.starts_with("Content-Length: ") {
                content_length = header_line["Content-Length: ".len()..].trim().parse().unwrap();
            }
        }
        
        // Read content
        if content_length > 0 {
            let mut buffer = vec![0; content_length];
            stdin_lock.read_exact(&mut buffer).unwrap();
            let content = String::from_utf8_lossy(&buffer);
            
            writeln!(log_file, "Received message: {}", content)
                .expect("Failed to write to log file");
            
            // Echo back the same message (for testing purposes)
            write!(stdout_lock, "Content-Length: {}\r\n\r\n{}", content.len(), content)
                .expect("Failed to write to stdout");
            stdout_lock.flush().unwrap();
            
            writeln!(log_file, "Sent response")
                .expect("Failed to write to log file");
        }
    }
}