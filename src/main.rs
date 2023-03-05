use futures::{executor, future, stream::SplitSink, SinkExt, StreamExt};
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use sysinfo::{CpuExt, System, SystemExt};
use tokio;
use warp::{
    ws::{Message, WebSocket, Ws},
    Filter, Rejection, Reply,
};

// create a type alias for the client vector
type ClientVec = Arc<Mutex<Vec<SplitSink<WebSocket, Message>>>>;

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

    // create routes
    let ui = warp::path!().and(warp::fs::dir("ui/dist/"));
    let ws = warp::path!("ws")
        .and(warp::ws())
        .and_then(move |ws: Ws| ws_handler(ws, clients.clone()));

    // combine routes
    let router = ui.or(ws);

    // start server
    warp::serve(router).run(([127, 0, 0, 1], 3000)).await;
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
