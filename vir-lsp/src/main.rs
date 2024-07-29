use virdant::parse::parse_package;
use serde_json::{Value, json};
use directories;
use log::*;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;


type Uri = String;

struct State {
    buffers: HashMap<Uri, Buffer>,
    shutdown: bool,
}

struct Buffer {
    uri: Uri,
    text: String,
}


pub fn main() {
    init_logging();
    std::panic::set_hook(Box::new(panic_handler));

    let (message_send, message_recv) = channel::<Value>();
    let _thread = std::thread::spawn(move || send_loop(message_send));

    recv_loop(message_recv);
}

fn send_loop(message_send: Sender<Value>) -> ! {
    loop {
        let message = read_message();
        message_send.send(message).unwrap();
    }
}

fn recv_loop(message_recv: Receiver<Value>) {
    let mut state = State::new();

    while !state.shutdown {
        match message_recv.recv() {
            Ok(message) => {
                info!("Handling message:\n{}", serde_json::to_string_pretty(&message).unwrap());
                state.handle_message(message);
            },
            Err(_e) => return,
        }
    }
}


impl State {
    fn new() -> Self {
        State {
            buffers: HashMap::new(),
            shutdown: false,
        }
    }

    fn handle_message(&mut self, message: Value) {
        match message.get("method") {
            None => warn!("No method on message."),
            Some(method) => {
                let method = method.as_str().unwrap();
                info!("Method: {method:?}");
                match method {
                    "initialize" => self.initialize(message),
                    "initialized" => (),
                    "shutdown" => self.shutdown = true,
                    "textDocument/didOpen" => self.text_document_did_open(message),
                    "textDocument/didChange" => self.text_document_did_change(message),
                    "textDocument/didSave" => self.text_document_did_save(message),
                    _ => warn!("Unhandled method: {method:?}"),
                }
            },
        }
    }

    fn initialize(&mut self, request: Value) {
        let response: Value = json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "capabilities": {
                    "positionEncoding": "utf-8",
                    "textDocumentSync": 1, // TextDocumentSyncKind.FULL
                },
            },
        });

        send_message(response);
    }

    fn buffer(&mut self, uri: &Uri) -> &mut Buffer {
        self.buffers.get_mut(uri).unwrap_or_else(|| {
            error!("No such URI: {uri}");
            panic!("No such URI: {uri}")
        })
    }

    fn open_buffer(&mut self, uri: &Uri, text: &str) -> &mut Buffer {
        info!("Opened buffer: {uri}");
        let buffer = Buffer::new(uri, text);
        self.buffers.insert(uri.clone(), buffer);
        self.buffers.get_mut(uri).unwrap_or_else(|| {
            error!("No such URI: {uri}");
            panic!("No such URI: {uri}")
        })
    }

    fn text_document_did_open(&mut self, message: Value) {
        let text = message["params"]["textDocument"]["text"].as_str().unwrap();
        let uri = message["params"]["textDocument"]["uri"].as_str().unwrap();
        let buffer = self.open_buffer(&uri.to_string(), text);
        warn!("text is {text:?}");
        buffer.send_diagnostics();
    }

    fn text_document_did_change(&mut self, message: Value) {
        let uri = message["params"]["textDocument"]["uri"].as_str().unwrap().to_string();
        let buffer = self.buffer(&uri);
        buffer.update_text(message["params"]["contentChanges"][0]["text"].as_str().unwrap().to_string());
        buffer.send_diagnostics();
    }

    fn text_document_did_save(&mut self, message: Value) {
        let uri = message["params"]["textDocument"]["uri"].as_str().unwrap().to_string();
        let buffer = self.buffer(&uri);
        buffer.send_diagnostics();
    }
}

impl Buffer {
    fn new(uri: &Uri, text: &str) -> Buffer {
        Buffer {
            uri: uri.to_string(),
            text: text.to_string(),
        }
    }

    fn update_text(&mut self, text: String) {
        debug!("{}", self.text);
        self.text = text.to_string();
    }

    fn send_diagnostics(&mut self) {
        let mut diagnostics = vec![];

        if let Err(err) = parse_package(&self.text) {
            let span = err.span();

            let start_line = span.start().line() - 1;
            let start_character = span.start().line() + 1;

            let end_line = span.end().line() - 1;
            let end_character = span.end().col() + 1;

            let message = err.message();

            let diagnostic = json!({
                "range": {
                    "start": { "line": start_line, "character": start_character },
                    "end": { "line": end_line, "character": end_character },
                },
                "severity": 1, // ERROR
                "message": message,
            });
            diagnostics.push(diagnostic);
        }

        let message = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": self.uri.to_string(),
                "diagnostics": diagnostics,
            },
        });
        send_message(message);
    }
}

fn panic_handler(info: &std::panic::PanicInfo) {
    error!("Panic occurred: {}", info);
}

fn init_logging() {
    let basedirs = directories::BaseDirs::new().unwrap();
    let virdant_dir = basedirs.config_local_dir().join("virdant");
    std::fs::create_dir_all(&virdant_dir).unwrap();
    let file = std::fs::File::create(virdant_dir.join("lsp.log")).unwrap();

    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env)
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    info!("LSP Started");
}

fn read_message() -> Value {
    use std::io::Read;
    use std::io::BufRead;
    let mut stdin = std::io::stdin().lock();

    let mut buffer = String::new();

    stdin.read_line(&mut buffer).unwrap();
    assert!(buffer.starts_with("Content-Length: "));
    let length = buffer.split(": ").collect::<Vec<_>>()[1].trim().parse::<usize>().unwrap();

    // throw away empty line
    stdin.read_line(&mut buffer).unwrap();
    let mut buffer: Vec<u8> = vec![0; length];
    let mut bytes_read = 0;
    while bytes_read < length {
        bytes_read += stdin.read(&mut buffer[bytes_read..]).unwrap();
    }
    assert_eq!(bytes_read, length);
    assert_eq!(buffer.len(), length);
    let buffer = String::from_utf8(buffer).unwrap();
    let message: Value = serde_json::from_str(&buffer).unwrap();
    let method = &message["method"];

    info!("RECEIVED:\n{}", serde_json::to_string_pretty(&method).unwrap());
    message
}

fn send_message(message: Value) {
    use std::io::Write;

    let value_str = serde_json::to_string_pretty(&message).unwrap();
    let value_str_len = value_str.len();

    print!("Content-Length: {value_str_len}\r\n\r\n{value_str}");
    std::io::stdout().flush().unwrap();

    info!("SENT: {}", serde_json::to_string_pretty(&message).unwrap());
}
