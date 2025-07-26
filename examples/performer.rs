use std::sync::{Arc, Mutex};
use tesse_rs::performer::TerminalPerformer;
use tesse_rs::screen::Screen;
use tracing::{Level, info};
use vte::Parser;

// Import tracing and its subscriber for setup
use tracing_subscriber::{self, EnvFilter, fmt}; // Use BufWriter for buffered file writes
fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .init();
    info!("Tesse terminal multiplexer starting..."); // Using info! macro
    let screen = Arc::new(Mutex::new(Screen::new(5, 10)));
    let mut perf = TerminalPerformer::new(screen.clone());
    let mut parser = Parser::new();

    // Sample data: print Hello, move cursor to 3,5, print X, clear screen
    let seqs: Vec<&[u8]> = vec![b"Hello, world!\n", b"\x1B[3;5H", b"X", b"\x1B[2J"];

    for seq in seqs {
        // feed entire sequence at once
        parser.advance(&mut perf, seq);

        let lines = screen.lock().unwrap().to_lines();
        println!("Screen state after sequences:");
        for (i, line) in lines.iter().enumerate() {
            println!("{:02}: '{}'", i, line);
        }
    }
}
