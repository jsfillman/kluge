use kluge_check::{lex, resolve, AdmitView, FsView};
use kluge_controller::{mint_webhook, Run, State};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

#[test]
fn bundle_is_the_ca_and_the_listener_handshakes() {
    let material = mint_webhook();
    assert!(material.ca_pem.contains("BEGIN CERTIFICATE"));
    assert!(material.cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(!material.ca_bundle.contains('\n'));
    assert_eq!(
        decode_base64(&material.ca_bundle).unwrap(),
        material.ca_pem.as_bytes()
    );
    let service = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../deploy/kluge/templates/service.yaml"
    ))
    .unwrap();
    assert!(service.contains("8443"), "{service}");
    let addr = kluge_controller::open_tls("127.0.0.1:0", &material, webhook_state()).unwrap();
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
    let mut tcp = TcpStream::connect(addr).unwrap();
    let mut tls = rustls::Stream::new(&mut connection, &mut tcp);
    tls.write_all(b"GET /admit HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut buf = [0u8; 64];
    let n = tls.read(&mut buf).unwrap_or(0);
    assert!(n > 0 || connection.peer_certificates().is_some());
}

fn webhook_state() -> std::sync::Arc<std::sync::Mutex<State>> {
    std::sync::Arc::new(std::sync::Mutex::new(State {
        context: Run {
            spec: resolve(&[], &lex("run t\nfor 1h\n").unwrap(), &[]).unwrap(),
            view: AdmitView {
                allow: Vec::new(),
                require_digest: false,
                pull_ok: true,
                live_names: Vec::new(),
                teams: Vec::new(),
                caller: Some("ada".to_string()),
                max_wall_secs: 3600,
            },
            nodes: Vec::new(),
            quota: 1,
            fs: FsView {
                root: "/".to_string(),
                existing: vec!["/".to_string()],
                links: Vec::new(),
            },
        },
        runs: Vec::new(),
        cluster: kluge_controller::MemoryCluster::default(),
        apiserver: None,
    }))
}

fn decode_base64(text: &str) -> Result<Vec<u8>, ()> {
    fn val(byte: u8) -> Result<u8, ()> {
        match byte {
            b'A'..=b'Z' => Ok(byte - b'A'),
            b'a'..=b'z' => Ok(byte - b'a' + 26),
            b'0'..=b'9' => Ok(byte - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(()),
        }
    }
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        let (a, b, c, d) = (bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]);
        let (a, b) = (val(a)?, val(b)?);
        out.push((a << 2) | (b >> 4));
        if c != b'=' {
            let c = val(c)?;
            out.push((b << 4) | (c >> 2));
            if d != b'=' {
                out.push((c << 6) | val(d)?);
            }
        }
        i += 4;
    }
    Ok(out)
}
