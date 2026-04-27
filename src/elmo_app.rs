use chrono::{DateTime, Local, Utc};
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::{
    layout::Constraint,
    style::Style,
    text::Text,
    widgets::{Row, StatefulWidget, Table, TableState, Widget},
    DefaultTerminal,
};
use std::time::Duration;

use crate::elos;

#[derive(Debug)]
pub struct ElmoApp<'a> {
    event_table: EventTable<'a>,
    event_details: EventDetails,
}

#[derive(Debug)]
pub struct ElmoState<'a> {
    event_table_state: EventTableState<'a>,
    event_details_state: EventDetailsState,
}

impl<'a> ElmoApp<'a> {
    pub fn new() -> Self {
        ElmoApp {
            event_table: EventTable {
                saticfy_rustc: Row::new([""]),
            },
            event_details: EventDetails {},
        }
    }
}

impl<'a> StatefulWidget for ElmoApp<'a> {
    type State = ElmoState<'a>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        self.event_table
            .render(area, buf, &mut state.event_table_state);
        if state.event_details_state.visible {
            self.event_details.render(
                area.centered(Constraint::Percentage(50), Constraint::Percentage(40)),
                buf,
                &mut state.event_details_state,
            );
        }
    }
}

impl<'a> ElmoApp<'a> {
    pub fn run(connection: &String, terminal: &mut DefaultTerminal) -> () {
        let mut state = ElmoState {
            event_table_state: EventTableState {
                table_state: TableState::default(),
                rows: vec![],
            },
            event_details_state: EventDetailsState {
                event: "N/A".to_string(),
                visible: false,
            },
        };

        let mut elos = match elos::Elos::connect_with(connection.clone()) {
            Ok(elos) => elos,
            Err(e) => {
                panic!("Failed to connect: {}", e);
            }
        };

        match elos.subscribe(&".e.classification 256 NE".to_string()) {
            Ok(_) => (),
            Err(e) => panic!("failed to subscribe: {}", e),
        }

        match elos.find_events(&"1 1 EQ".to_string()) {
            Ok(events) => events.iter().for_each(|ev| {
                state.event_table_state.rows.push(Row::new([
                    format_severity(ev.severity),
                    ev.messageCode.unwrap_or(0).to_string(),
                    format_timespec(ev.date),
                    ev.payload.clone().unwrap_or("".to_string()),
                ]))
            }),
            Err(e) => panic!("failed to fetch historical events: {}", e),
        }

        loop {
            elos.read_event_queue(*elos.subscribtions().get(0).unwrap())
                .map(|events| {
                    events.iter().for_each(|ev| {
                        state.event_table_state.rows.push(Row::new([
                            format_severity(ev.severity),
                            ev.messageCode.unwrap_or(0).to_string(),
                            format_timespec(ev.date),
                            ev.payload.clone().unwrap_or("".to_string()),
                        ]))
                    });
                })
                .unwrap();
            terminal
                .draw(|frame| {
                    frame.render_stateful_widget(ElmoApp::new(), frame.area(), &mut state)
                })
                .unwrap();
            if event::poll(Duration::from_millis(250)).unwrap() {
                match event::read().unwrap() {
                    event::Event::Key(event) => match event.code {
                        KeyCode::Char('q') => {
                            if state.event_details_state.visible == true {
                                state.event_details_state.visible = false
                            } else {
                                break;
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            state.event_table_state.table_state.select_next()
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            state.event_table_state.table_state.select_previous()
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            state.event_table_state.table_state.select_next_column()
                        }

                        KeyCode::Char('h') | KeyCode::Left => {
                            state.event_table_state.table_state.select_previous_column()
                        }

                        KeyCode::Char('g') | KeyCode::Home => {
                            state.event_table_state.table_state.select_first()
                        }
                        KeyCode::Char('G') | KeyCode::End => {
                            state.event_table_state.table_state.select_last()
                        }
                        KeyCode::Enter => {
                            if state.event_details_state.visible == false {
                                let selected =
                                    state.event_table_state.table_state.selected().unwrap();

                                state.event_details_state.event =
                                    format!("index: {}", selected.to_string());
                                state.event_details_state.visible = true;
                            }

                            ()
                        }
                        _ => {}
                    },
                    event => {
                        println!("event {:?}", event)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct EventTable<'a> {
    saticfy_rustc: Row<'a>,
}

#[derive(Debug)]
struct EventTableState<'a> {
    table_state: TableState,
    rows: Vec<Row<'a>>,
}

impl<'a> EventTableState<'a> {
    pub fn add_row(&mut self, fields: [String; 5]) {
        self.rows.push(Row::new(fields));
    }
}

impl<'a> StatefulWidget for EventTable<'a> {
    type State = EventTableState<'a>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let header =
            Row::new(["Severity", "Message Code", "Date", "payload"]).style(Style::new().bold());
        let footer = Row::new([
            format!("Events {}", state.rows.len()),
            "Filtered".to_string(),
        ]);
        let widths = [
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ];
        let mut rows_sorted = state.rows.clone();
        rows_sorted.reverse();

        let table = Table::new(rows_sorted, widths)
            .header(header)
            .footer(footer)
            .row_highlight_style(Style::new().on_black().bold());

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

#[derive(Debug)]
struct EventDetails {}

#[derive(Debug)]
struct EventDetailsState {
    event: String,
    visible: bool,
}

impl StatefulWidget for EventDetails {
    type State = EventDetailsState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let event_popup = Block::bordered().title("Event Details");
        Widget::render(Clear, area, buf);
        event_popup.render(area, buf);
        Text::from(state.event.clone()).render(
            Rect {
                x: area.x + 2,
                y: area.y + 1,
                width: area.width - 4,
                height: 2,
            },
            buf,
        );
    }
}

fn format_severity(severity: Option<u32>) -> String {
    match severity {
        Some(1) => "☠".to_string(),
        Some(2) => "❌".to_string(),
        Some(3) => "⚠️".to_string(),
        Some(4) => "💡".to_string(),
        Some(5) => "🪲".to_string(),
        Some(6) => "🗣".to_string(),
        _ => "".to_string(),
    }
}

fn format_timespec(ts: [i64; 2]) -> String {
    match DateTime::<Utc>::from_timestamp(ts[0] as i64, ts[1] as u32 / 100) {
        Some(date) => date
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string(),
        None => "n/a".to_string(),
    }
}

fn event_to_row(event: &elos::Event) -> Row {
    Row::new([
        format_severity(event.severity),
        event.messageCode.unwrap_or(0).to_string(),
        format_timespec(event.date),
        event.payload.clone().unwrap_or("".to_string()),
    ])
}
