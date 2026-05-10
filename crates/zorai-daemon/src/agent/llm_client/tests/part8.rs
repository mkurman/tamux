use super::*;
#[tokio::test]
async fn anthropic_response_header_request_id_is_forwarded_on_done_chunk() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_response_id\",\"usage\":{\"input_tokens\":3}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"text\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"hello\"}}\n\n",
        "data: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":5}}\n\n",
        "data: {\"type\":\"message_stop\"}\n\n"
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let body = body.to_string();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nrequest-id: req_stream_123\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write response");
    });

    let response = client
        .get(format!("http://{addr}"))
        .send()
        .await
        .expect("send test request");
    let request_id = response
        .headers()
        .get("request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let (tx, mut rx) = mpsc::channel(8);
    parse_anthropic_sse(response, request_id, &tx)
        .await
        .expect("parse should succeed");
    drop(tx);

    let mut done_chunk = None;
    while let Some(chunk) = rx.recv().await {
        if let CompletionChunk::Done { request_id, .. } = chunk.expect("chunk") {
            done_chunk = Some(request_id);
            break;
        }
    }

    assert_eq!(done_chunk, Some(Some("req_stream_123".to_string())));
    server.await.expect("server task");
}
