use crate::elos::{Elos, Event};
use crate::widgets::event_view::EventView;
use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub counter: usize,
    pub elos: Option<Elos>,
    pub events: Vec<Event>,
    pub event_view: EventView,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            elos: None,
            events: vec![],
            event_view: EventView::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        App {
            elos:Elos::connect().ok(),
            ..Default::default()
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }

    pub fn read_events(&mut self) {
        let _ = self.elos.as_mut().map(|elos| {
            elos.read_all_event_queues()
                .map(|events| {
                    self.events.extend(events);
                    self.counter = self.events.len();
                })
                .map_err(|err| eprint!("fail to fetch events: {}", err))
        });
    }
}
