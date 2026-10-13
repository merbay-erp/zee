mod contracts;
use postgres::{fallible_iterator::FallibleIterator, types::ToSql, Client, NoTls};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "contracts") { contracts::run(); return; }
    let rows: usize = args[1].parse().unwrap();
    let bytes: usize = args[2].parse().unwrap();
    assert!(rows <= 128 && bytes <= 64 * 1024 * 1024);
    let mut client = Client::connect(&std::env::var("ZEE_RSS_URL").unwrap(), NoTls).unwrap();
    let sql = format!("SELECT repeat('x', {bytes})::text AS payload FROM generate_series(1,{rows})");
    let params: Vec<(&dyn ToSql, postgres::types::Type)> = Vec::new();
    let mut iter = client.query_typed_raw(&sql, params).unwrap();
    let mut owned = Vec::new();
    let mut total = 0usize;
    let mut exceeded = false;
    while let Some(row) = iter.next().unwrap() {
        if exceeded { continue; }
        let value: &str = row.try_get(0).unwrap();
        total = total.saturating_add("payload".len() + value.len());
        if total > 16 * 1024 * 1024 {
            exceeded = true;
            owned.clear();
        } else {
            owned.push(value.to_owned());
        }
    }
    drop(iter);
    if exceeded { println!("error=PostgreSQL sonucu 16 MiB sınırını aşıyor"); }
    else { println!("ok_rows={}", owned.len()); }
    println!("next_query_ok={}", client.query("SELECT 'ok'::text AS health", &[]).is_ok());
}
