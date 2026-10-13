use dil::{postgresql::PostgresqlOturumu, veritabani_modeli::{VeritabaniBildirimi, VeritabaniHedefi, VeritabaniHatasi, VeritabaniSatirlari}};
use postgres::{Client, Row, NoTls, types::{Type, ToSql}, fallible_iterator::FallibleIterator};
fn error(message: impl Into<String>, data: Vec<(String,String)>) -> VeritabaniHatasi { VeritabaniHatasi{mesaj:message.into(),veri:data} }
fn pg_error(prefix:&str,e:postgres::Error)->VeritabaniHatasi {
 let mut data=Vec::new();let message=if let Some(db)=e.as_db_error(){data.push(("sqlstate".into(),db.code().code().into()));if let Some(c)=db.constraint(){data.push(("kısıt".into(),c.into()));}if let Some(t)=db.table(){data.push(("tablo".into(),t.into()));}format!("{prefix}: {}",db.message())}else{prefix.to_string()};error(message,data)
}
fn convert(row:&Row,total:&mut usize)->Result<Vec<(String,String)>,VeritabaniHatasi>{
 if row.len()>100{return Err(error("PostgreSQL sonucu 100 sütun sınırını aşıyor",vec![]));}
 let mut fields=Vec::with_capacity(row.len());
 for (i,column) in row.columns().iter().enumerate(){
  let name=column.name();
  if !matches!(*column.type_(),Type::TEXT|Type::VARCHAR|Type::BPCHAR|Type::NAME){return Err(error(format!("\"{name}\" sütunu Metin değil; sorguda açık ::text dönüşümü kullan"),vec![("sutun".into(),name.into())]));}
  let value:Option<&str>=row.try_get(i).map_err(|e|pg_error("PostgreSQL sonuç sütunu okunamadı",e))?;
  let value=value.ok_or_else(||error(format!("\"{name}\" sütunu NULL; sorguda COALESCE ile açık metin değeri seç"),vec![("sutun".into(),name.into())]))?;
  *total=total.saturating_add(name.len()+value.len());
  if *total>16*1024*1024{return Err(error("PostgreSQL sonucu 16 MiB sınırını aşıyor",vec![]));}
  fields.push((name.to_string(),value.to_string()));
 }
 Ok(fields)
}
fn collect(client:&mut Client,sql:&str)->Result<VeritabaniSatirlari,VeritabaniHatasi>{
 let params:Vec<(&dyn ToSql,Type)>=Vec::new();
 let mut rows=client.query_typed_raw(sql,params).map_err(|e|pg_error("PostgreSQL sorgusu başarısız",e))?;
 let mut count=0usize;let mut total=0usize;let mut saved_error=None;let mut result=Vec::new();
 while let Some(row)=rows.next().map_err(|e|pg_error("PostgreSQL sorgusu başarısız",e))?{
  count=count.saturating_add(1);
  if count>10_000 {saved_error=Some(error("PostgreSQL sonucu 10.000 satır sınırını aşıyor",vec![]));result.clear();continue;}
  if saved_error.is_some(){continue;}
  match convert(&row,&mut total){Ok(fields)=>result.push(fields),Err(e)=>{saved_error=Some(e);result.clear();}}
 }
 match saved_error{Some(e)=>Err(e),None=>Ok(result)}
}
pub fn run(){
 let target=std::env::var("ZEE_RSS_TARGET").unwrap();let config=std::env::var("ZEE_RSS_URL").unwrap();
 let binding=VeritabaniBildirimi{hedef:VeritabaniHedefi::ayristir(&target).unwrap(),baglanti_degiskeni:"ZEE_RSS_URL".into(),gocler:"unused".into(),istek_baglami_cozucusu:None};
 let mut old=PostgresqlOturumu::baglan(&binding,0).unwrap();let mut candidate=Client::connect(&config,NoTls).unwrap();
 let mut queries=vec!["SELECT 'ok'::text AS value".to_string(),"SELECT NULL::text AS value WHERE FALSE".into(),"SELECT NULL::text AS value".into(),"SELECT 1::integer AS value".into(),"SELECT 'x'::varchar AS value, 'a'::char(2) AS padded, 'n'::name AS named".into(),"SELECT i::text AS value FROM generate_series(1,10000) i".into(),"SELECT i::text AS value FROM generate_series(1,10001) i".into(),"SELECT NULL::text AS value FROM generate_series(1,10001) i".into(),"SELECT i::integer AS value FROM generate_series(1,10001) i".into(),"SELECT repeat('x',1048576)::text AS payload FROM generate_series(1,64)".into(),"SELECT repeat('x',2048)::text AS payload FROM generate_series(1,10001)".into(),"SELECT CASE WHEN i=10 THEN (1/(i-i))::text ELSE NULL::text END AS value FROM generate_series(1,10) i".into(),"SELECT CASE WHEN i=10002 THEN (1/(i-i))::text ELSE 'x'::text END AS value FROM generate_series(1,10002) i".into()];
 for columns in [100,101]{queries.push(format!("SELECT {}",(0..columns).map(|i|format!("'x'::text AS c{i}")).collect::<Vec<_>>().join(",")));}
 for (i,q) in queries.iter().enumerate(){assert_eq!(old.oku(q,&[]),collect(&mut candidate,q),"query {}",i+1);assert!(old.oku("SELECT 'ok'::text AS health",&[]).is_ok());assert!(collect(&mut candidate,"SELECT 'ok'::text AS health").is_ok());}
 old.eylem_baslat().unwrap();candidate.batch_execute("BEGIN").unwrap();let q="SELECT repeat('x',1048576)::text AS payload FROM generate_series(1,64)";assert_eq!(old.oku(q,&[]),collect(&mut candidate,q));assert_eq!(old.oku("SELECT 'ok'::text AS health",&[]),collect(&mut candidate,"SELECT 'ok'::text AS health"));old.eylem_geri_al().unwrap();candidate.batch_execute("ROLLBACK").unwrap();
 println!("contract_queries={} transaction_health=passed original_Zee_vs_stream=equal transport_error_precedence=passed",queries.len());
}
