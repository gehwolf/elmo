mod elos;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Row, Table, TableState};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "localhost:54321")]
    connection: String,
}

struct App {
    args: Args,
    table_state: TableState,
}

fn main() {
    let _ = ratatui::run(|terminal| App::new().run(terminal));
}

impl App {
    fn new() -> Self {
        Self {
            args: Args::parse(),
            table_state: TableState::default(),
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> () {
        let mut rows = vec![];
        let mut elos = match elos::Elos::connect_with(self.args.connection) {
            Ok(elos) => elos,
            Err(e) => {
                panic!("Failed to connect: {}", e);
            }
        };

        let result = elos.send(&elos::Message {
            version: 0x1,
            command: 0x1,
            length: 0,
            data: vec![],
        });
        match result {
            Ok(_) => (),
            Err(e) => panic!("failed to send: {}", e),
        }

        match elos.receive() {
            Ok(msg) => (),
            Err(e) => panic!("failed receive: {}", e),
        }

        match elos.subscribe(&".e.classification 256 NE".to_string()) {
            Ok(_) => (),
            Err(e) => panic!("failed to subscribe: {}", e),
        }

        match elos.find_events(&"1 1 EQ".to_string()) {
            Ok(events) => events.iter().for_each(|ev| {
                // println!("--> {}", ev);

                rows.push(Row::new([
                    format_severity(ev.severity),
                    format!("{:?}", ev.messageCode),
                    format_timespec(ev.date),
                    format!("{:?}", ev.payload),
                ]))
            }),
            Err(e) => panic!("failed to fetch historical events: {}", e),
        }

        loop {
            let _ =
                terminal.draw(|frame| render(frame, &mut elos, &mut rows, &mut self.table_state));

            if event::poll(Duration::from_millis(250)).unwrap() {
                match event::read().unwrap() {
                    event::Event::Key(event) => match event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') | KeyCode::Down => self.table_state.select_next(),
                        KeyCode::Char('k') | KeyCode::Up => self.table_state.select_previous(),
                        KeyCode::Char('l') | KeyCode::Right => {
                            self.table_state.select_next_column()
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            self.table_state.select_previous_column()
                        }
                        KeyCode::Char('g') | KeyCode::Home => self.table_state.select_first(),
                        KeyCode::Char('G') | KeyCode::End => self.table_state.select_last(),
                        _ => {}
                    },
                    event => {
                        println!("event {:?}", event)
                    }
                }
            }
        }
        let _ = elos.disconnect();

        ()
    }
}

fn render(
    frame: &mut Frame,
    elos: &mut elos::Elos,
    rows: &mut Vec<Row>,
    table_state: &mut TableState,
) {
    let header =
        Row::new(["Severity", "Message Code", "Date", "payload"]).style(Style::new().bold());

    elos.read_event_queue(*elos.subscribtions().get(0).unwrap())
        .map(|events| {
            events.iter().for_each(|ev| {
                rows.push(Row::new([
                    format_severity(ev.severity),
                    format!("{:?}", ev.messageCode),
                    format_timespec(ev.date),
                    format!("{:?}", ev.payload),
                ]))
            });
        })
        .unwrap();

    let footer = Row::new([format!("Events {}", rows.len()), "Filtered".to_string()]);
    let widths = [
        Constraint::Percentage(10),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(50),
    ];
    let mut rows_sorted = rows.clone();
    rows_sorted.reverse();

    let table = Table::new(rows_sorted, widths)
        .header(header)
        .footer(footer)
        .row_highlight_style(Style::new().on_black().bold());

    frame.render_stateful_widget(
        table,
        Rect {
            x: frame.area().x,
            y: frame.area().y,
            width: frame.area().width,
            height: frame.area().height,
        },
        table_state,
    );
}

fn format_severity(severity: Option<u32>) -> String {
    match severity {
        Some(1) => "☠".to_string(),
        Some(2) => "❌".to_string(),
        Some(3) => "⚠️".to_string(),
        Some(4) => "💡".to_string(),
        Some(5) => "🐛".to_string(),
        Some(6) => "🗣".to_string(),
        _ => "".to_string(),
    }
}

fn format_timespec(ts: [u32; 2]) -> String {
    DateTime::<Utc>::from_timestamp(ts[0] as i64, ts[1] / 100)
        .unwrap()
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S%.3f")
        .to_string()
}
