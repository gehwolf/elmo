use byteorder::{NetworkEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::io::Result;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};

pub struct Elos {
    stream: TcpStream,
    subscribtions: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub date: [i64; 2],
    pub messageCode: Option<u32>,
    pub classification: Option<u64>,
    pub severity: Option<u32>,
    pub payload: Option<String>,
}

pub struct Message {
    pub version: u8,
    pub command: u8,
    pub length: u16,
    pub data: Vec<u8>,
}

#[derive(Deserialize)]
struct PublishResponse {
    error: Option<String>,
}

#[derive(Deserialize)]
struct SubscribeResponse {
    error: Option<String>,
    eventQueueIds: Vec<u64>,
}

#[derive(Deserialize)]
struct ReadEventQueueResponse {
    error: Option<String>,
    eventArray: Vec<Event>,
}

#[derive(Deserialize)]
struct LogFindEventResponse {
    error: Option<String>,
    eventArray: Vec<Event>,
    isTruncated: Option<bool>,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{date: {:?}, messageCode: {:?}, classification: {:?}, severity: {:?}, payload: {:?} }}",
            self.date,
            self.messageCode,
            self.classification,
            self.severity,
            self.payload,
        )
    }
}

impl Elos {
    pub fn connect() -> Result<Elos> {
        Elos::connect_with("localhost:54321".to_string())
    }

    pub fn connect_with(connection: String) -> Result<Elos> {
        TcpStream::connect(connection).map(|stream| Elos {
            stream,
            subscribtions: vec![],
        })
    }

    pub fn disconnect(&self) -> Result<()> {
        self.stream.shutdown(Shutdown::Both)
    }

    pub fn send(&mut self, msg: &Message) -> Result<()> {
        msg.serialize(&mut self.stream)
    }

    pub fn receive(&mut self) -> Result<Box<Message>> {
        Message::deserialize(&mut self.stream)
    }

    pub fn subscribe(&mut self, filter: &String) -> Result<()> {
        let request = json!({
            "filter": [filter],
        });
        let message = Message {
            version: 0x01,
            command: 0x03,
            length: 0,
            data: request.to_string().into_bytes(),
        };
        self.send(&message)?;
        self.receive().map(|response| {
            let json_string: String = String::from_utf8(response.data).unwrap().to_owned();
            #[cfg(elos_debug)]
            println!("subscribe: '{}'", json_string);
            serde_json::from_str(&json_string).map(|subscribe_response: SubscribeResponse| {
                match subscribe_response.error {
                    Some(error) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Protocol error: {}", error),
                    )),
                    None => {
                        self.subscribtions
                            .extend_from_slice(&subscribe_response.eventQueueIds);
                        Ok(())
                    }
                }
            })?
        })?
    }

    pub fn find_events(&mut self, filter: &String) -> Result<Vec<Event>> {
        let request = json!({
            "filter": filter,
        });
        let mut message = Message {
            version: 0x01,
            command: 0x04,
            length: 0,
            data: request.to_string().into_bytes(),
        };
        let mut events: Vec<Event> = vec![];

        loop {
            self.send(&message)?;
            let response = self.receive()?;
            let json_string: String = String::from_utf8(response.data).unwrap().to_owned();
            #[cfg(elos_debug)]
            println!("find event : {}", json_string);

            let mut log_find_event_response: LogFindEventResponse =
                serde_json::from_str(&json_string).expect("invalid json response");

            match log_find_event_response.error {
                Some(error) => return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Protocol error: {}", error),
                )),
                _ => {
                    events.append(&mut log_find_event_response.eventArray);
                    match log_find_event_response.isTruncated {
                        Some(false)| _ => break,
                        Some(true) => {
                            message.data = json!({"filter":filter, "oldest": events.last().unwrap().date}).to_string().into_bytes();
                        }
                    }
                }
            };
        }
        Ok(events)
    }

    pub fn read_event_queue(&mut self, event_queue_id: u64) -> Result<Vec<Event>> {
        let request = json!({
            "eventQueueId": event_queue_id,
        });
        let message = Message {
            version: 0x01,
            command: 0x05,
            length: 0,
            data: request.to_string().into_bytes(),
        };
        self.send(&message)?;
        self.receive().map(|response| {
            let json_string: String = String::from_utf8(response.data).unwrap().to_owned();
            #[cfg(elos_debug)]
            println!("read event queue : {}", json_string);
            serde_json::from_str(&json_string).map(
                |read_event_queue_response: ReadEventQueueResponse| match read_event_queue_response
                    .error
                {
                    Some(error) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Protocol error: {}", error),
                    )),
                    None => Ok(read_event_queue_response.eventArray),
                },
            )?
        })?
    }

    pub fn read_all_event_queues(&mut self) -> Result<Vec<Event>> {
        let mut all_events = vec![];
        self.subscribtions.clone().into_iter().for_each(|sub_id| {
            self.read_event_queue(sub_id)
                .map_or_else(|_err| (), |events| all_events.extend(events));
        });
        Ok(all_events)
    }

    pub fn publish(&mut self, event: Event) -> Result<()> {
        let message = Message {
            version: 0x01,
            command: 0x02,
            length: 0,
            data: json!(event).to_string().into_bytes(),
        };
        self.send(&message)?;
        self.receive().map(|response| {
            let json_string: String = String::from_utf8(response.data).unwrap().to_owned();
            #[cfg(elos_debug)]
            println!("read event queue : {}", json_string);
            serde_json::from_str(&json_string).map(|publish_response: PublishResponse| {
                match publish_response.error {
                    Some(error) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Protocol error: {}", error),
                    )),
                    None => Ok(()),
                }
            })?
        })?
    }

    pub fn subscribtions(&self) -> &Vec<u64> {
        &self.subscribtions
    }
}

impl Message {
    pub fn new() -> Message {
        Message {
            version: 0,
            command: 0,
            length: 0,
            data: Vec::new(),
        }
    }

    pub fn serialize(&self, buffer: &mut impl Write) -> Result<()> {
        let length = (self.data.len() as u16).to_be();
        #[cfg(elos_debug)]
        println!("msglength {:04x}", length);
        let header = vec![
            self.version,
            self.command,
            ((length & 0xFF00) >> 8) as u8,
            (length & 0x00FF) as u8,
        ];
        buffer.write_all(header.as_slice())?;
        buffer.write_all(self.data.as_slice())?;
        Ok(())
    }

    pub fn deserialize(buffer: &mut impl Read) -> Result<Box<Message>> {
        let version = buffer.read_u8()?;
        let command = buffer.read_u8()?;
        let length = buffer.read_u16::<NetworkEndian>()?;
        #[cfg(elos_debug)]
        println!(
            "receive version: {:x}, command: {:x}, length:{:x}",
            version,
            command,
            length.to_be()
        );
        let mut message = Box::new(Message {
            version,
            command,
            length: length.to_be(),
            data: vec![0; length.to_be() as usize],
        });
        buffer.read_exact(&mut message.data)?;
        message
            .data
            .iter()
            .position(|&c| c == 0)
            .map(|pos| message.data.truncate(pos));
        Ok(message)
    }
}

#[test]
fn test_serialize() {
    let message = Message {
        version: 0x1,
        command: 0x2,
        length: 17,
        data: b"{hugo hat husten}".to_vec(),
    };
    let length = message.data.len();
    let mut bytes: Vec<u8> = vec![];
    match message.serialize(&mut bytes) {
        Ok(_) => println!("message send: {}", String::from_utf8(message.data).unwrap()),
        Err(e) => println!("Failed to connect: {}", e),
    }
    println!("{:02X?}", bytes);
    assert_eq!(
        bytes.len(),
        length + 4,
        "Ensure expected overall message size"
    );
    assert_eq!(bytes[0], message.version, "Verify version is set");
    assert_eq!(bytes[1], message.command, "Verify command is set");
    assert_eq!(
        bytes[3],
        (message.length.to_be() & 0x00FF) as u8,
        "Verify correct payload length is set {}",
        (message.length.to_be() & 0x00FF) as u8
    );
    assert_eq!(
        bytes[2],
        ((message.length.to_be() & 0xFF00) >> 8) as u8,
        "Verify correct payload length is set {}",
        ((message.length.to_be() & 0xFF00) >> 8) as u8
    );
}

#[test]
fn test_send() {
    let payload = b"{\"payload\":\"hugo hat husten\"}";
    let elos = Elos::connect();
    match elos {
        Ok(mut conn) => {
            println!("Successfully connected to server on port 54321");
            let message = Message {
                version: 0x1,
                command: 0x2,
                length: payload.len() as u16,
                data: payload.to_vec(),
            };

            match conn.send(&message) {
                Ok(_) => println!("message send: {}", String::from_utf8(message.data).unwrap()),
                Err(e) => println!("Failed to connect: {}", e),
            }
            let _ = conn.disconnect();
        }
        Err(e) => {
            assert!(false, "Failed to connect: {}", e);
        }
    }
}

#[test]
fn test_receive() {
    let json_response: String;
    let elos = Elos::connect();
    match elos {
        Ok(mut conn) => {
            println!("Successfully connected to server on port 54321");
            let message = Message {
                version: 0x1,
                command: 0x1,
                length: 0,
                data: vec![],
            };

            match conn.send(&message) {
                Ok(_) => println!("message send: {}", String::from_utf8(message.data).unwrap()),
                Err(e) => println!("Failed to connect: {}", e),
            }

            match conn.receive() {
                Ok(msg) => {
                    json_response = String::from_utf8(msg.data).unwrap();
                    println!("response: {}", json_response);
                    assert_eq!(msg.command, 0x81);
                    assert_ne!(msg.length, 0x0);
                }
                Err(e) => println!("Failed to connect: {}", e),
            }
            let _ = conn.disconnect();
        }
        Err(e) => {
            assert!(false, "Failed to connect: {}", e);
        }
    }
}

#[test]
fn test_subscribe() {
    let elos = Elos::connect();
    match elos {
        Ok(mut conn) => {
            conn.subscribe(&"1 1 EQ".to_string()).unwrap();
            assert_eq!(
                conn.subscribtions.len(),
                1,
                "Subscriptions shall contain exactly one"
            );
            conn.disconnect().unwrap();
        }
        Err(e) => {
            assert!(false, "Failed to connect: {}", e);
        }
    }
}

#[test]
fn test_fetch_event_queue() {
    let elos = Elos::connect();
    match elos {
        Ok(mut conn) => {
            conn.subscribe(&"1 1 EQ".to_string()).unwrap();

            conn.publish(Event {
                messageCode: Some(1001),
                payload: Some("Hugo hat husten".to_owned()),
                date: [120, 121],
                classification: Some(0x1),
                severity: Some(1),
            })
            .unwrap();
            conn.read_event_queue(*conn.subscribtions.get(0).unwrap())
                .map(|events| {
                    assert_ne!(events.len(), 0, "At least one event shall be retrieved");
                })
                .unwrap();

            for i in 0..5 {
                conn.publish(Event {
                    messageCode: Some(1001),
                    payload: Some(format!("Hugo hat husten {}", i)),
                    date: [120, 121],
                    classification: Some(0x1),
                    severity: Some(1),
                })
                .unwrap();
            }
            conn.read_event_queue(*conn.subscribtions.get(0).unwrap())
                .map(|events| {
                    assert_eq!(events.len(), 5, "5 events shall be retrieved");
                })
                .unwrap();

            conn.disconnect().unwrap();
        }
        Err(e) => {
            assert!(false, "Failed to connect: {}", e);
        }
    }
}
