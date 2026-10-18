use std::sync::Arc;

use crate::log::{LogMutex, Record};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
    routing::{get, post},
    serve, Router,
};
use serde::{Deserialize, Serialize};

//Avoid shared mutable state
#[derive(Clone)]
struct ServerState {
    log: Arc<LogMutex>,
}

#[derive(Deserialize)]
struct ProduceRequest {
    value: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
struct ConsumeResponse {
    data: Vec<u8>,
    offset: Option<usize>,
}

#[derive(Serialize)]
enum Error {
    GenericError
}

pub async fn init(addr: &str) {
    async fn consume_handler(
        State(state): State<ServerState>,
        Path(offset): Path<usize>,
    ) -> impl IntoResponse {
        let result = state.log.read(offset);
        match result {
            Ok(ok) => Json(ConsumeResponse {
                data: ok.value,
                offset: ok.offset,
            })
            .into_response(),
            Err(_) => Json(Error::GenericError).into_response(),
        }
    }

    async fn produce_handler(
        State(state): State<ServerState>,
        Json(req): Json<ProduceRequest>,
    ) -> impl IntoResponse {
        let result = state.log.append(Record {
            offset: None,
            value: req.value,
        });
        match result {
            Ok(ok) => Json(ok).into_response(),
            Err(_) => Json(Error::GenericError).into_response(),
        }
    }
    let state = ServerState {
        log: Arc::new(LogMutex::new()),
    };

    let app = Router::new()
        .route("/consume/{offset}", get(consume_handler))
        .route("/produce", post(produce_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}
