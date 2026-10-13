use postgres::{Client, NoTls};
use postgres_protocol::message::backend::FrameLimitExceeded;
use std::error::Error;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bytes: usize = args[1].parse().unwrap();
    assert!(bytes <= 64 * 1024 * 1024);
    let transaction = args[2] == "txn";
    let mut client = Client::connect(&std::env::var("ZEE_RSS_URL").unwrap(), NoTls).unwrap();
    if transaction {
        client
            .batch_execute("BEGIN; INSERT INTO zee_frame_probe VALUES ('before')")
            .unwrap();
    }
    let sql = format!("SELECT repeat('x',{bytes})::text AS payload");
    match client.query(&sql, &[]) {
        Ok(rows) => {
            assert!(bytes <= 16 * 1024 * 1024);
            assert_eq!(rows[0].get::<_, String>(0).len(), bytes);
            println!("accepted_bytes={bytes}");
        }
        Err(error) => {
            assert!(bytes >= 32 * 1024 * 1024);
            let mut cause = error.source();
            let mut matched = false;
            while let Some(inner) = cause {
                let limit = inner.downcast_ref::<FrameLimitExceeded>().or_else(|| {
                    inner
                        .downcast_ref::<std::io::Error>()
                        .and_then(std::io::Error::get_ref)
                        .and_then(|cause| cause.downcast_ref::<FrameLimitExceeded>())
                });
                if let Some(limit) = limit {
                    assert_eq!(limit.announced, bytes + 10);
                    assert_eq!(limit.limit, 32 * 1024 * 1024);
                    matched = true;
                    println!("typed_limit={} announced={}", limit.limit, limit.announced);
                }
                cause = inner.source();
            }
            assert!(matched, "typed cause lost in driver error chain: {error}");
        }
    }
    let closed = client.is_closed();
    assert_eq!(closed, bytes >= 32 * 1024 * 1024);
    let health = client.query("SELECT 'ok'::text AS health", &[]).is_ok();
    assert_eq!(health, !closed);
    println!("closed={closed} next_query_ok={health}");
    if transaction {
        assert!(client.batch_execute("ROLLBACK").is_err());
        println!("rollback_acknowledged=false");
    }
}
