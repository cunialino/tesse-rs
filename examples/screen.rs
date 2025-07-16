use tesse_rs::screen::Screen;

fn main() {
    let mut screen = Screen::new(3, 8);
    print_state(&screen, "Initial");

    screen.put_char('A');
    screen.put_char('B');
    print_state(&screen, "After AB");

    screen.line_feed();
    screen.put_char('C');
    print_state(&screen, "After C on next line");

    screen.resize(2, 5);
    print_state(&screen, "After resize to 2×5");

    screen.clear();
    print_state(&screen, "After clear");
}

fn print_state(screen: &Screen, label: &str) {
    println!("-- {} --", label);
    for line in screen.to_lines() {
        println!("'{}'", line);
    }
    println!();
}

