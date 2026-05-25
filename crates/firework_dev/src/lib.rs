// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::thread;

use firework_adapter::{AdapterClickPhase, AdapterCommand, AdapterEvent, AdapterResult};

static NEXT_PROXY_ID: AtomicUsize = AtomicUsize::new(1);
static WRITE_STREAM: Mutex<Option<TcpStream>> = Mutex::new(None);

static SYNC_RESPONSE_RX: Mutex<Option<mpsc::Receiver<String>>> = Mutex::new(None);
static SYNC_RESPONSE_TX: Mutex<Option<mpsc::Sender<String>>> = Mutex::new(None);

fn init_sync_channel() {
    let mut rx_lock = SYNC_RESPONSE_RX.lock().unwrap();
    if rx_lock.is_none() {
        let (tx, rx) = mpsc::channel();
        *SYNC_RESPONSE_TX.lock().unwrap() = Some(tx);
        *rx_lock = Some(rx);
    }
}

fn get_write_stream() -> Option<TcpStream> {
    let mut lock = WRITE_STREAM.lock().unwrap();
    if lock.is_none() {
        init_sync_channel();

        match TcpStream::connect("127.0.0.1:9090") {
            Ok(stream) => {
                stream.set_nodelay(true).unwrap();
                let read_stream = stream.try_clone().unwrap();
                *lock = Some(stream);

                spawn_reader_thread(read_stream, None);
            }

            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        }
    }

    lock.as_ref().unwrap().try_clone().ok()
}

fn spawn_reader_thread(stream: TcpStream, listener: Option<fn(AdapterEvent)>) {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        let tx = SYNC_RESPONSE_TX.lock().unwrap().as_ref().unwrap().clone();

        for line in reader.lines() {
            if let Ok(msg) = line {
                if msg.contains("\"res\":") {
                    // Это синхронный ответ
                    let _ = tx.send(msg);
                } else if let Some(cb) = listener {
                    // Аинхронное событие
                    if msg.contains("\"evt\":\"Tick\"") {
                        cb(AdapterEvent::Tick);
                    } else if msg.contains("\"evt\":\"Touch\"") {
                        // Примитивный парсинг чтобы не тянуть зависимости, Dev ориентирован только
                        // на скорость компиляции
                        let x = extract_int(&msg, "\"x\":").unwrap_or(0);
                        let y = extract_int(&msg, "\"y\":").unwrap_or(0);
                        let p = extract_int(&msg, "\"phase\":").unwrap_or(3);

                        let phase = match p {
                            0 => AdapterClickPhase::Began,
                            1 => AdapterClickPhase::Moved,
                            2 => AdapterClickPhase::Ended,
                            _ => AdapterClickPhase::Cancelled,
                        };

                        cb(AdapterEvent::Touch(x as u32, y as u32, phase));
                    }
                }
            }
        }
    });
}

fn extract_int(json: &str, key: &str) -> Option<i32> {
    let start = json.find(key)? + key.len();
    let end = json[start..]
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(json.len() - start);
    json[start..start + end].parse().ok()
}

fn send_json(json: String) {
    if let Some(mut stream) = get_write_stream() {
        let _ = stream.write_all(json.as_bytes());
        let _ = stream.write_all(b"\n");
    }
}

fn wait_for_response() -> Option<String> {
    let rx_lock = SYNC_RESPONSE_RX.lock().unwrap();
    if let Some(rx) = rx_lock.as_ref() {
        return rx.recv().ok();
    }
    None
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub fn network_adapter<'a>(cmd: AdapterCommand<'a>) -> AdapterResult {
    match cmd {
        AdapterCommand::RemoveAll => {
            send_json(r#"{"cmd":"RemoveAll"}"#.to_string());
            AdapterResult::Void
        }
        AdapterCommand::RunLoop { listener, .. } => {
            if let Some(stream) = get_write_stream() {
                spawn_reader_thread(stream.try_clone().unwrap(), Some(listener));
                send_json(r#"{"cmd":"RunLoop"}"#.to_string());
            }
            AdapterResult::Void
        }
        AdapterCommand::Render => AdapterResult::Void,
        AdapterCommand::NewRect { .. } => {
            let id = NEXT_PROXY_ID.fetch_add(1, Ordering::SeqCst);
            send_json(format!(r#"{{"cmd":"NewRect","id":{}}}"#, id));
            AdapterResult::Handle(id)
        }
        AdapterCommand::NewText { .. } => {
            let id = NEXT_PROXY_ID.fetch_add(1, Ordering::SeqCst);
            send_json(format!(r#"{{"cmd":"NewText","id":{}}}"#, id));
            AdapterResult::Handle(id)
        }
        AdapterCommand::SetPosition(id, (x, y)) => {
            send_json(format!(
                r#"{{"cmd":"SetPosition","id":{},"pos":[{},{}]}}"#,
                id, x, y
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetSize(id, (w, h)) => {
            send_json(format!(
                r#"{{"cmd":"SetSize","id":{},"size":[{},{}]}}"#,
                id, w, h
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetColor(id, (r, g, b, a)) => {
            send_json(format!(
                r#"{{"cmd":"SetColor","id":{},"color":[{},{},{},{}]}}"#,
                id, r, g, b, a
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetZ(id, z) => {
            send_json(format!(r#"{{"cmd":"SetZ","id":{},"z":{}}}"#, id, z));
            AdapterResult::Void
        }
        AdapterCommand::SetVisible(id, vis) => {
            send_json(format!(
                r#"{{"cmd":"SetVisible","id":{},"vis":{}}}"#,
                id, vis
            ));
            AdapterResult::Void
        }
        AdapterCommand::Remove(id) => {
            send_json(format!(r#"{{"cmd":"Remove","id":{}}}"#, id));
            AdapterResult::Void
        }
        AdapterCommand::SetHitGroup(id, group) => {
            send_json(format!(
                r#"{{"cmd":"SetHitGroup","id":{},"group":{}}}"#,
                id, group
            ));
            AdapterResult::Void
        }
        AdapterCommand::ResolveHit(group, (x, y, w, h)) => {
            send_json(format!(
                r#"{{"cmd":"ResolveHit","group":{},"rect":[{},{},{},{}]}}"#,
                group, x, y, w, h
            ));
            if let Some(resp) = wait_for_response() {
                let id = extract_int(&resp, "\"id\":").unwrap_or(-1);
                if id != -1 {
                    return AdapterResult::Handle(id as usize);
                }
            }
            AdapterResult::Void
        }
        AdapterCommand::PushText { handle, text, mode } => {
            send_json(format!(
                r#"{{"cmd":"PushText","id":{},"mode":{},"text":"{}"}}"#,
                handle,
                mode,
                escape_json_string(text)
            ));
            AdapterResult::Void
        }
        AdapterCommand::ClearText(id) => {
            send_json(format!(r#"{{"cmd":"ClearText","id":{}}}"#, id));
            AdapterResult::Void
        }
        AdapterCommand::MeasureText(id) => {
            send_json(format!(r#"{{"cmd":"MeasureText","id":{}}}"#, id));
            if let Some(resp) = wait_for_response() {
                let w = extract_int(&resp, "\"w\":").unwrap_or(0);
                let h = extract_int(&resp, "\"h\":").unwrap_or(0);
                return AdapterResult::Size(w as u32, h as u32);
            }
            AdapterResult::Size(0, 0)
        }
        AdapterCommand::SetTextAlign(id, align) => {
            send_json(format!(
                r#"{{"cmd":"SetTextAlign","id":{},"align":{}}}"#,
                id, align
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetTextWrapWidth(id, width) => {
            send_json(format!(
                r#"{{"cmd":"SetTextWrapWidth","id":{},"width":{}}}"#,
                id, width
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetCornerRadius(id, (tl, tr, br, bl)) => {
            send_json(format!(
                r#"{{"cmd":"SetCornerRadius","id":{},"radius":[{},{},{},{}]}}"#,
                id, tl, tr, br, bl
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetBorder(id, width, (r, g, b, a)) => {
            send_json(format!(
                r#"{{"cmd":"SetBorder","id":{},"width":{},"color":[{},{},{},{}]}}"#,
                id, width, r, g, b, a
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetFontSize(id, size) => {
            send_json(format!(
                r#"{{"cmd":"SetFontSize","id":{},"size":{}}}"#,
                id, size
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetShadow(id, (dx, dy), blur, (r, g, b, a)) => {
            send_json(format!(
                r#"{{"cmd":"SetShadow","id":{},"offset":[{},{}],"blur":{},"color":[{},{},{},{}]}}"#,
                id, dx, dy, blur, r, g, b, a
            ));
            AdapterResult::Void
        }
        AdapterCommand::SetClipTo(id, clip_id) => {
            send_json(format!(
                r#"{{"cmd":"SetClipTo","id":{},"clip_id":{}}}"#,
                id, clip_id
            ));
            AdapterResult::Void
        }
    }
}
