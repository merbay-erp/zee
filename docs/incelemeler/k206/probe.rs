use dil::postgresql::PostgresqlOturumu;
use dil::veritabani_modeli::{VeritabaniBildirimi, VeritabaniHedefi};
fn main() {
 let a: Vec<String> = std::env::args().collect();
 let n: usize = a[1].parse().unwrap(); let bytes: usize = a[2].parse().unwrap();
 assert!(n <= 128 && bytes <= 64*1024*1024);
 let target = std::env::var("ZEE_RSS_TARGET").unwrap();
 let b=VeritabaniBildirimi{hedef:VeritabaniHedefi::ayristir(&target).unwrap(),baglanti_degiskeni:"ZEE_RSS_URL".into(),gocler:"unused".into(),istek_baglami_cozucusu:None};
 let mut session=PostgresqlOturumu::baglan(&b,0).unwrap();
 let sql=format!("SELECT repeat('x', {bytes})::text AS payload FROM generate_series(1,{n})");
 match session.oku(&sql,&[]) {Ok(v)=>println!("ok_rows={}",v.len()),Err(e)=>println!("error={}",e.mesaj)}
 println!("next_query_ok={}",session.oku("SELECT 'ok'::text AS health",&[]).is_ok());
}
