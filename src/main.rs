use elos::{Elos, Event};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Terminal,
};
use std::io::{stdout, Result};

mod elos;

pub struct Elmo {
    counter: usize,
    exit: bool,
    elos: Elos,
    events: Vec<Event>,
}

fn main() -> Result<()> {
    let mut elmo: Elmo = Elmo {
        counter: 0,
        exit: false,
        elos: Elos::connect()?,
        events: vec![],
    };

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let _ = elmo.elos.subscribe(&"1 1 EQ".to_owned());

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(
                Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                    .white()
                    .on_blue(),
                area,
            );

            let counter_text = Text::from(vec![Line::from(vec![
                "Value: ".into(),
                elmo.counter.to_string().yellow(),
            ])]);
            frame.render_widget(Paragraph::new(counter_text).white().on_blue(), area);
            let mut events_text: Vec<Line> = vec![];
            for event in &elmo.events {
                events_text.push(Line::from(vec![
                    event.messageCode.unwrap_or(0).to_string().into(),
                    event.payload.clone().unwrap_or("".to_owned()).into(),
                ]));
            }
            frame.render_widget(Paragraph::new(events_text).white().on_blue(), area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    elmo.exit = true;
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('r') {
                    let _ = elmo
                        .elos
                        .read_all_event_queues()
                        .map(|events| {
                            elmo.events.extend(events);
                            elmo.counter = elmo.events.len();
                        })
                        .map_err(|err| eprint!("fail to fetch events: {}", err));
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('+') {
                    if elmo.counter < 255 {
                        elmo.counter += 1;
                    }
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('-') {
                    if elmo.counter > 0 {
                        elmo.counter -= 1;
                    }
                }
            }
        }
        if elmo.exit {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
