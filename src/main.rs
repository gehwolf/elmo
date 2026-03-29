mod elos;
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{List, Paragraph, Row, Table};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use std::time::Duration;
use chrono::{DateTime, NaiveDateTime, Utc};

fn main() {
    let _ = ratatui::run(run);
}

fn run(terminal: &mut DefaultTerminal) -> () {
    let mut rows = vec![];
    let mut elos = match elos::Elos::connect() {
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
        Ok(_) => println!("message send"),
        Err(e) => panic!("failed to send: {}", e),
    }

    match elos.receive() {
        Ok(msg) => println!("response : {}", String::from_utf8(msg.data).unwrap()),
        Err(e) => panic!("failed receive: {}", e),
    }

    match elos.subscribe(&"1 1 EQ".to_string()) {
        Ok(_) => (),
        Err(e) => panic!("failed to subscribe: {}", e),
    }

    loop {
        let _ = terminal.draw(|frame| render(frame, &mut elos, &mut rows));

        if event::poll(Duration::from_millis(250)).unwrap() {
            match event::read().unwrap() {
                event::Event::Key(event) => {
                    if event.code == KeyCode::Char('q') {
                        break;
                    }
                }
                event => {
                    println!("event {:?}", event)
                }
            }
        }
    }
    let _ = elos.disconnect();

    ()
}

fn render(frame: &mut Frame, elos: &mut elos::Elos, rows: &mut Vec<Row>) {
    let header = Row::new(["Severity", "Date", "Source", "payload"]).style(Style::new().bold());

    elos.read_event_queue(*elos.subscribtions().get(0).unwrap())
        .map(|events| {
            events.iter().for_each(|ev| {
                rows.push(Row::new([
                    format_severity( ev.severity),
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
        .footer(footer);

    frame.render_widget(
        table,
        Rect {
            x: frame.area().x,
            y: frame.area().y,
            width: frame.area().width,
            height: frame.area().height,
        },
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


fn format_timespec(ts: [u32;2]) -> String {
    let naive = NaiveDateTime::from_timestamp_opt(ts[0]as i64, ts[1] as u32)
        .expect("invalid timestamp");
    let dt: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    dt.format("%Y-%m-%d %H:%M:%S%.f UTC").to_string()
}

