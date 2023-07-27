use crate::ha_api::HAApi;
use crate::teams_states::TeamsStates;
use crate::utils;
use futures_util::{future, pin_mut, StreamExt};
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time;
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const ENV_API_TOKEN: &str = "TSAPITOKEN";

pub struct TeamsAPI {
    pub teams_states: Arc<TeamsStates>,
    pub url: String,
}

impl TeamsAPI {
    pub fn new() -> Self {
        let api_token = utils::get_env_var(ENV_API_TOKEN);
        let teams_states = Arc::new(TeamsStates {
            camera_on: AtomicBool::new(false),
            in_meeting: AtomicBool::new(false),
        });

        let url = format!(
            "ws://localhost:8124?token={}&protocol-version=1.0.0",
            api_token
        );

        Self { teams_states, url }
    }
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}

pub async fn parse_data(json: &str, listener: Arc<HAApi>, teams_states: Arc<TeamsStates>) {
    let answer = json::parse(&json.to_string()).unwrap();
    let new_in_meeting = answer["meetingUpdate"]["meetingState"]["isInMeeting"]
        .as_bool()
        .expect("Unable to locate isInMeeting variable in JSON");
    let new_camera_on = answer["meetingUpdate"]["meetingState"]["isCameraOn"]
        .as_bool()
        .expect("Unable to locate isCameraOn variable in JSON");

    if (teams_states
        .in_meeting
        .swap(new_in_meeting, Ordering::Relaxed)
        != new_in_meeting)
        || (teams_states
            .camera_on
            .swap(new_in_meeting, Ordering::Relaxed)
            != new_camera_on)
    {
        listener.notify_changed(&teams_states).await;
    }
}

pub async fn start_listening(
    listener: Arc<HAApi>,
    teams_states: Arc<TeamsStates>,
    is_running: Arc<AtomicBool>,
    url: String,
) {
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));
    let url_local = url::Url::parse(&url).unwrap();
    let (ws_stream, _) = connect_async(url_local).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();
    let stdin_to_ws = stdin_rx.map(Ok).forward(write); // this here finishes right away
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = &message.unwrap().into_data();
            let json = String::from_utf8_lossy(data);
            info!("{}", json);
            parse_data(&json, listener.clone(), teams_states.clone()).await;
        })
    };

    let running_future = async {
        let one_second = time::Duration::from_secs(1);

        while is_running.load(Ordering::Relaxed) {
            sleep(one_second);
        }
        info!("Application close requested");
    };

    pin_mut!(stdin_to_ws, running_future, ws_to_stdout);
    let ws_futures = async {
        future::select(stdin_to_ws, ws_to_stdout).await;
    };
    pin_mut!(ws_futures);
    future::select(ws_futures, running_future).await;
}

// mod tests {
//     #[test]
//     #[should_panic(expected = "TSAPITOKEN")]
//     fn new_missing_api_key_will_panic() {
//         std::env::set_var(ENV_API_TOKEN, "");
//         TeamsAPI::new(Arc::new(DefaultListener {}));
//     }
//
//     #[test]
//     fn listen_test() {
//         let mut api = TeamsAPI::new(Arc::new(DefaultListener {}));
//         api.listen_loop();
//     }
// }
