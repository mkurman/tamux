use super::*;
#[tokio::test]
async fn anthropic_stream_stop_metadata_is_forwarded_on_done_chunk() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stop_meta\",\"usage\":{\"input_tokens\":2}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"text\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"done\"}}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"stop_sequence\",\"stop_sequence\":\"END\"},\"usage\":{\"output_tokens\":4}}\n\n",
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
            content,
            input_tokens,
            output_tokens,
            stop_reason,
            stop_sequence,
            ..
        } = chunk.expect("chunk")
        {
            done_chunk = Some((
                content,
                input_tokens,
                output_tokens,
                stop_reason,
                stop_sequence,
            ));
            break;
        }
    }

    assert_eq!(
        done_chunk,
        Some((
            "done".to_string(),
            2,
            4,
            Some("stop_sequence".to_string()),
            Some("END".to_string()),
        ))
    );
    server.await.expect("server task");
}

#[tokio::test]
async fn anthropic_stream_stop_metadata_is_forwarded_on_tool_call_chunk() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stop_tool\",\"usage\":{\"input_tokens\":2}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"web_search\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"query\\\":\\\"cats\\\"}\"}}\n\n",
        "data: {\"type\":\"content_block_stop\"}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":1}}\n\n",
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

    let mut tool_chunk = None;
    while let Some(chunk) = rx.recv().await {
        if let CompletionChunk::ToolCalls {
            tool_calls,
            input_tokens,
            output_tokens,
            stop_reason,
            stop_sequence,
            ..
        } = chunk.expect("chunk")
        {
            tool_chunk = Some((
                tool_calls.len(),
                input_tokens,
                output_tokens,
                stop_reason,
                stop_sequence,
            ));
            break;
        }
    }

    assert_eq!(
        tool_chunk,
        Some((1, Some(2), Some(1), Some("tool_use".to_string()), None,))
    );
    server.await.expect("server task");
}

#[tokio::test]
async fn anthropic_stream_model_is_forwarded_on_done_chunk() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_model\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-sonnet-4-6\",\"container\":{\"id\":\"container_123\",\"expires_at\":\"2026-04-05T12:00:00Z\"},\"usage\":{\"input_tokens\":1}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"text\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}\n\n",
        "data: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":2}}\n\n",
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
            content,
            upstream_model,
            upstream_role,
            upstream_message_type,
            upstream_container,
            ..
        } = chunk.expect("chunk")
        {
            done_chunk = Some((
                content,
                upstream_model,
                upstream_role,
                upstream_message_type,
                upstream_container.map(|value| (value.id, value.expires_at)),
            ));
            break;
        }
    }

    assert_eq!(
        done_chunk,
        Some((
            "ok".to_string(),
            Some("claude-sonnet-4-6".to_string()),
            Some("assistant".to_string()),
            Some("message".to_string()),
            Some((
                "container_123".to_string(),
                "2026-04-05T12:00:00Z".to_string(),
            )),
        ))
    );
    server.await.expect("server task");
}

#[tokio::test]
async fn anthropic_stream_done_chunk_carries_upstream_message_text_blocks() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_upstream\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-sonnet-4-6\",\"usage\":{\"input_tokens\":1}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"text\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"hello upstream\"}}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":2}}\n\n",
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
            upstream_message, ..
        } = chunk.expect("chunk")
        {
            done_chunk = upstream_message;
            break;
        }
    }

    let upstream_message = done_chunk.expect("upstream message");
    assert_eq!(upstream_message.id.as_deref(), Some("msg_upstream"));
    assert_eq!(upstream_message.message_type.as_deref(), Some("message"));
    assert_eq!(upstream_message.role.as_deref(), Some("assistant"));
    assert_eq!(upstream_message.model.as_deref(), Some("claude-sonnet-4-6"));
    assert_eq!(upstream_message.stop_reason.as_deref(), Some("end_turn"));
    assert_eq!(upstream_message.content_blocks.len(), 1);
    assert_eq!(upstream_message.content_blocks[0].block_type, "text");
    assert_eq!(
        upstream_message.content_blocks[0].text.as_deref(),
        Some("hello upstream")
    );
    server.await.expect("server task");
}

#[tokio::test]
async fn anthropic_stream_tool_calls_chunk_carries_upstream_message_tool_use_blocks() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_tool_upstream\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-sonnet-4-6\",\"usage\":{\"input_tokens\":1}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"web_search\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"query\\\":\\\"cats\\\"}\"}}\n\n",
        "data: {\"type\":\"content_block_stop\"}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":1}}\n\n",
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

    let mut tool_chunk = None;
    while let Some(chunk) = rx.recv().await {
        if let CompletionChunk::ToolCalls {
            upstream_message, ..
        } = chunk.expect("chunk")
        {
            tool_chunk = upstream_message;
            break;
        }
    }

    let upstream_message = tool_chunk.expect("upstream message");
    assert_eq!(upstream_message.id.as_deref(), Some("msg_tool_upstream"));
    assert_eq!(upstream_message.stop_reason.as_deref(), Some("tool_use"));
    assert_eq!(upstream_message.content_blocks.len(), 1);
    assert_eq!(upstream_message.content_blocks[0].block_type, "tool_use");
    assert_eq!(
        upstream_message.content_blocks[0].id.as_deref(),
        Some("toolu_1")
    );
    assert_eq!(
        upstream_message.content_blocks[0].name.as_deref(),
        Some("web_search")
    );
    assert_eq!(
        upstream_message.content_blocks[0].input_json,
        Some(serde_json::json!({"query": "cats"}))
    );
    server.await.expect("server task");
}

#[tokio::test]
async fn anthropic_stream_done_chunk_carries_upstream_message_thinking_blocks() {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_thinking_upstream\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-sonnet-4-6\",\"usage\":{\"input_tokens\":1}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"thinking\",\"signature\":\"sig_1\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"step by step\"}}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":2}}\n\n",
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
            upstream_message, ..
        } = chunk.expect("chunk")
        {
            done_chunk = upstream_message;
            break;
        }
    }

    let upstream_message = done_chunk.expect("upstream message");
    assert_eq!(upstream_message.content_blocks.len(), 1);
    assert_eq!(upstream_message.content_blocks[0].block_type, "thinking");
    assert_eq!(
        upstream_message.content_blocks[0].signature.as_deref(),
        Some("sig_1")
    );
    assert_eq!(
        upstream_message.content_blocks[0].thinking.as_deref(),
        Some("step by step")
    );
    server.await.expect("server task");
}
