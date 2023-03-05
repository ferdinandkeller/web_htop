use futures::{executor, future, stream::SplitSink, SinkExt, StreamExt};
use include_dir::{include_dir, Dir};
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use sysinfo::{CpuExt, System, SystemExt};
use tokio;
use warp::{
    path::FullPath,
    ws::{Message, WebSocket, Ws},
    Filter, Rejection, Reply,
};

// include the content of the ui/dist directory
// this is the compiled frontend produced by vite
static PROJECT_DIR: Dir = include_dir!("ui/dist");

// create a type alias for the client vector
type ClientVec = Arc<Mutex<Vec<SplitSink<WebSocket, Message>>>>;

// make main function async but single threaded
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // create a vector to store client websocket senders
    let clients: ClientVec = Arc::new(Mutex::new(vec![]));
    let clients_clone = clients.clone();

    // start cpu monitor in a separate thread
    thread::spawn(move || {
        // save a clone of the clients vector
        let clients = clients_clone;

        // create a new system monitor
        let mut sys = System::new();

        // refresh the system monitor until the program is closed
        loop {
            // refresh the system monitor's cpu data
            sys.refresh_cpu();

            // get the cpu usage for each cpu
            let cpus = sys
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect::<Vec<f32>>();

            // build the data into a JSON object
            let data = json!(cpus);

            // borrow the clients vector
            let mut clients_link = clients.lock().expect("Failed to lock clients vector");

            // create a vector of futures to send the data to each client
            let request = clients_link.iter_mut().map(|client| async {
                // send the data to the client
                client
                    .send(Message::text(data.to_string()))
                    .await
                    .expect("Failed to send data to client");
            });

            // execute all the futures at once
            executor::block_on(future::join_all(request));

            // wait 500ms before refreshing again
            thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    // create the ui route
    let ui = warp::path::full().map(|fullpath: FullPath| {
        // find the path of the requested file by removing the leading slash
        let mut path = &fullpath.as_str()[1..];

        // if the path is empty, set it to index.html
        if path == "" {
            path = "index.html";
        }

        // get the requested file entry if it exists
        // otherwise return a 404 response
        let entry = match PROJECT_DIR.get_entry(path) {
            Some(entry) => entry,
            None => {
                return warp::http::Response::builder()
                    .status(404)
                    .header("content-type", "text/text")
                    .body(b"404 Not Found" as &[u8]);
            }
        };

        match entry {
            // if the entry is a file, return the file contents
            include_dir::DirEntry::File(file) => {
                // get the mime type of the file
                let mime = match path.split(".").last() {
                    Some("html") => "text/html",
                    Some("js") => "text/javascript",
                    Some("css") => "text/css",
                    Some("ttf") => "font/ttf",
                    _ => panic!("azd"),
                };
                // get the content of the file
                let content = file.contents();
                // return a reply
                return warp::http::Response::builder()
                    .header("content-type", mime)
                    .body(content);
            }

            // if the entry is a directory, return a 404 response
            _ => {
                return warp::http::Response::builder()
                    .status(404)
                    .header("content-type", "text/text")
                    .body(b"404 Not Found" as &[u8]);
            }
        }
    });

    // create the websocket route
    let ws = warp::path!("ws")
        .and(warp::ws())
        .and_then(move |ws: Ws| ws_handler(ws, clients.clone()));

    // combine the routes
    let router = ws.or(ui);

    // print link in the console
    println!("Server running at http://localhost:3133");

    // start server
    warp::serve(router).run(([127, 0, 0, 1], 3133)).await;
}

// handle new websocket connections
async fn ws_handler(ws: Ws, clients: ClientVec) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(|socket: WebSocket| client_connection(socket, clients)))
}

// handle websocket messages
async fn client_connection(socket: WebSocket, clients: ClientVec) {
    // split the socket into a sender and receiver
    let (socket_sender, _) = socket.split();
    // add the sender to the client vector
    clients.lock().expect("").push(socket_sender);
}
