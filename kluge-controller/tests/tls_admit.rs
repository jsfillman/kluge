use kluge_check::{lex, resolve, AdmitView, FsView};
use kluge_controller::{mint_webhook, open_tls, Run, State};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

fn state() -> std::sync::Arc<std::sync::Mutex<State>> {
    Arc::new(std::sync::Mutex::new(State {
        context: Run {
            spec: resolve(&[], &lex("run t\nfor 1h\n").unwrap(), &[]).unwrap(),
            view: AdmitView {
                allow: vec!["registry.example.com/".to_string()],
                require_digest: false,
                pull_ok: true,
                live_names: Vec::new(),
                teams: vec!["kernel-search".to_string()],
                caller: Some("ada".to_string()),
                max_wall_secs: 3600,
            },
            nodes: Vec::new(),
            quota: 64,
            fs: FsView {
                root: "/scratch/team".to_string(),
                existing: vec!["/scratch/team".to_string()],
                links: Vec::new(),
            },
        },
        runs: Vec::new(),
        cluster: kluge_controller::MemoryCluster::default(),
        apiserver: None,
    }))
}

#[test]
fn tls_admit_refuses_an_empty_pool() {
    let material = mint_webhook();
    let addr = open_tls("127.0.0.1:0", &material, state()).unwrap();
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut material.ca_pem.as_bytes()) {
        root_store.add(cert.unwrap()).unwrap();
    }
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let mut connection = rustls::ClientConnection::new(
        Arc::new(config),
        rustls::pki_types::ServerName::try_from("localhost").unwrap(),
    )
    .unwrap();
    let review = serde_json::json!({
        "request": {"uid": "abc", "object": {"spec": {"text": "run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n"}}}
    })
    .to_string();
    let request = format!(
        "POST /admit HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{review}",
        review.len()
    );
    let mut tcp = TcpStream::connect(addr).unwrap();
    let mut tls = rustls::Stream::new(&mut connection, &mut tcp);
    tls.write_all(request.as_bytes()).unwrap();
    let mut raw = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        match tls.read(&mut tmp) {
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&tmp[..n]),
        }
    }
    let buf = String::from_utf8_lossy(&raw);
    let body = buf.split_once("\r\n\r\n").map(|(_, body)| body).unwrap_or("");
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(parsed["response"]["uid"], "abc");
    assert_eq!(parsed["response"]["allowed"], false);
    assert!(
        parsed["response"]["status"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("pool.type"),
        "{body}"
    );
    let main = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs")).unwrap();
    assert!(main.contains("ca_bundle"), "{main}");
    assert!(main.contains("open_tls"), "{main}");
}
