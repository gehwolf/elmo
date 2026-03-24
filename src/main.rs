mod elos;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::widgets::Paragraph;
use signal_hook::{consts::SIGQUIT, consts::SIGTERM, iterator::Signals};
use std::{thread, time::Duration};

fn main() {
    let _ = ratatui::run(run);//context("ratatui failed");

    println!("Hello, world!");
}

fn run(terminal: &mut DefaultTerminal) -> () {
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

    let mut signals = Signals::new(&[SIGTERM, SIGQUIT]).unwrap();
    let mut quit = false;
    loop {
        elos.read_event_queue(*elos.subscribtions().get(0).unwrap())
            .map(|events| {
                events.iter().for_each(|ev| println!("event: {}", ev));
            })
            .unwrap();

        terminal.draw(render);
        for signal in signals.pending() {
            match signal {
                SIGQUIT | SIGTERM => quit = true,
                _ => (),
            }
        }

        match quit {
            false => thread::sleep(Duration::from_millis(500)),
            true => break,
        }
    }
    let _ = elos.disconnect();

    ()
}

fn render(frame: &mut Frame) {
        let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    frame.render_widget(greeting, frame.area());

}
