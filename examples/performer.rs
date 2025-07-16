use std::sync::{Arc, Mutex};
use my_crate::screen::Screen;   // replace `my_crate` with your crate name
use my_crate::parser::TerminalPerformer;
use vte::Parser;

fn main() {
    // Initialize a small screen
    let screen = Arc::new(Mutex::new(Screen::new(5, 10)));
    let mut perf = TerminalPerformer::new(screen.clone());
    let mut parser = Parser::new();

    // Sample data: print Hello, move cursor to 3,5, print X, clear screen
    let seqs: Vec<&[u8]> = vec![
        b"Hello, world!\n",
        b"\x1B[3;5H",
        b"X",
        b"\x1B[2J",
    ];

    for seq in seqs {
        // feed entire sequence at once
        parser.advance(&mut perf, seq);
    }

    // Dump screen lines
    let lines = screen.lock().unwrap().to_lines();
    println!("Screen state after sequences:");
    for (i, line) in lines.iter().enumerate() {
        println!("{:02}: '{}'", i, line);
    }
}
