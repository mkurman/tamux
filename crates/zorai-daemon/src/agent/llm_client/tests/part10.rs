use super::*;
#[tokio::test]
async fn anthropic_stream_usage_counters_are_forwarded_on_done_chunk() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_usage_stream\",\"usage\":{\"input_tokens\":3,\"cache_creation_input_tokens\":2}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"text\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"hello\"}}\n\n",
        "data: {\"type\":\"message_delta\",\"usage\":{\"input_tokens\":9,\"output_tokens\":5,\"cache_read_input_tokens\":7,\"server_tool_use\":{\"web_fetch_requests\":1,\"web_search_requests\":4}}}\n\n",
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
            "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
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

    let (tx, mut rx) = mpsc::channel(8);
    parse_anthropic_sse(response, None, &tx)
        .await
        .expect("parse should succeed");
    drop(tx);

    let mut done_chunk = None;
    while let Some(chunk) = rx.recv().await {
        if let CompletionChunk::Done {
            input_tokens,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            server_tool_use,
            ..
        } = chunk.expect("chunk")
        {
            let web_fetch_requests = server_tool_use
                .as_ref()
                .and_then(|value| value.web_fetch_requests);
            let web_search_requests = server_tool_use
                .as_ref()
                .and_then(|value| value.web_search_requests);
            done_chunk = Some((
                input_tokens,
                output_tokens,
                cache_creation_input_tokens,
                cache_read_input_tokens,
                web_fetch_requests,
                web_search_requests,
            ));
            break;
        }
    }

    assert_eq!(done_chunk, Some((9, 5, Some(2), Some(7), Some(1), Some(4))));
    server.await.expect("server task");
}
