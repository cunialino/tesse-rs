use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    error::Error,
    io::{self, ErrorKind, Read, Write},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::task;
use vte::Parser;
mod performer;
use performer::TerminalPerformer;

struct TerminalSession {
    pty: portable_pty::PtyPair,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    buffer: Arc<Mutex<Vec<String>>>,
    performer: Arc<Mutex<TerminalPerformer>>,
    parser: Arc<Mutex<Parser>>,
}

impl TerminalSession {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let pty_system = native_pty_system();
        let pty = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let mut cmd = CommandBuilder::new("bash");
        cmd.env("TERM", "xterm-256color");
        let _child = pty.slave.spawn_command(cmd)?;

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let performer = Arc::new(Mutex::new(TerminalPerformer::new(buffer.clone())));
        let parser = Arc::new(Mutex::new(Parser::new()));

        let writer = Arc::new(Mutex::new(pty.master.take_writer()?));

        Ok(Self {
            pty,
            writer,
            buffer,
            performer,
            parser,
        })
    }

    async fn start_reader(&mut self) -> Result<(), Box<dyn Error>> {
        let mut reader = self.pty.master.try_clone_reader()?;
        let perf = self.performer.clone();
        let pars = self.parser.clone();

        task::spawn_blocking(move || {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut parser = pars.lock().unwrap();
                        let mut p = perf.lock().unwrap();
                        parser.advance(&mut *p, &buf[..n]);
                        // p.flush_line();
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) => {
                        eprintln!("PTY read error: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn send_input(&mut self, input: &str) -> Result<(), Box<dyn Error>> {
        let mut w = self.writer.lock().unwrap();
        w.write_all(input.as_bytes())?;
        w.flush()?;
        Ok(())
    }

    fn get_display_lines(&self) -> Vec<String> {
        let mut lines = self.buffer.lock().unwrap().clone();
        if let Some(current) = self.performer.lock().unwrap().current_line.clone().into() {
            lines.push(current);
        }
        lines
    }
}

struct App {
    session: TerminalSession,
    should_quit: bool,
}

impl App {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let mut session = TerminalSession::new().await?;
        session.start_reader().await?;
        Ok(Self {
            session,
            should_quit: false,
        })
    }

    async fn handle_key(&mut self, key: KeyCode) -> Result<(), Box<dyn Error>> {
        match key {
            KeyCode::Char(c) => {
                self.session.send_input(&c.to_string()).await?;
            }
            KeyCode::Enter => {
                self.session.send_input("\n").await?;
            }
            KeyCode::Backspace => {
                self.session.send_input("\x7f").await?;
            }
            KeyCode::Tab => {
                self.session.send_input("\t").await?;
            }
            KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
        Ok(())
    }

    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(f.area());

        let lines = self.session.get_display_lines();
        let text: Vec<Line> = lines
            .into_iter()
            .map(|l| Line::from(Span::raw(l)))
            .collect();

        let term = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Terminal"))
            .style(Style::default().fg(Color::White));
        f.render_widget(term, chunks[0]);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await?;

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(ev) = event::read()? {
                app.handle_key(ev.code).await?;
            }
        }

        terminal.draw(|f| app.render(f))?;
        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
