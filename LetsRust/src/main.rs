#[tokio::main]
async fn main() {
    server::http::init("0.0.0.0:3000").await;
}
