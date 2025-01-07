use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            app.counter
        ))
        .block(
            Block::bordered()
                .title("Template")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .centered(),
        frame.area(),
    );

    let mut events_text: Vec<Line> = vec![];
    for event in &app.events {
        events_text.push(Line::from(vec![
            event.messageCode.unwrap_or(0).to_string().into(),
            event.payload.clone().unwrap_or("".to_owned()).into(),
        ]));
    }
    frame.render_widget(Paragraph::new(events_text).white().on_blue(), frame.area());
    app.event_view.render(&mut app, &mut frame);
}
