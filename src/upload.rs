use axum::{extract::{State, Multipart}, http::StatusCode, Json};
use csv::StringRecord;
use tokio_postgres::types::ToSql;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::Client;
use crate::db::DbPool;
use std::sync::Arc;
use serde_json::json;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

pub async fn upload_accounts(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(internal)? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let data = field.bytes().await.map_err(internal)?;
            file_bytes = Some(data.to_vec());
            break;
        }
    }
    let Some(bytes) = file_bytes else {
        return Err((StatusCode::BAD_REQUEST, "multipart field 'file' required".into()));
    };

    // Parse CSV header quickly (no full-file buffering later)
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_slice());

    let header = rdr.headers().map_err(|_| bad_req("missing header"))?.clone();
    let idx = map_header(&header, &["asof_date","account_id","sec_id","qty"])
        .ok_or_else(|| bad_req("required columns: asof_date, account_id, sec_id, qty"))?;

    // Single DB connection + transaction
    let pool = &state.db.0;
    let mut client = pool.get().await.map_err(internal)?;
    let tr = client.transaction().await.map_err(internal)?;

    // Temp staging table
    tr.execute(
        "CREATE TEMP TABLE t_upload_accounts(
           asof_date date not null,
           account_id text not null,
           sec_id text not null,
           qty double precision not null
         ) ON COMMIT DROP;", &[]
    ).await.map_err(internal)?;

    // Binary COPY for speed
    {
        let sink = tr.copy_in("COPY t_upload_accounts (asof_date, account_id, sec_id, qty) FROM STDIN BINARY").await.map_err(internal)?;
        let mut writer = BinaryCopyInWriter::new(sink, &[Date, Text, Text, Float8]);

        // Iterate rows streaming
        for rec in rdr.records() {
            let rec: StringRecord = rec.map_err(|e| bad_req(&format!("csv read error: {e}")))?;
            let asof = rec.get(idx["asof_date"]).unwrap().trim();
            let account = rec.get(idx["account_id"]).unwrap().trim();
            let sec = rec.get(idx["sec_id"]).unwrap().trim();
            let qty = rec.get(idx["qty"]).unwrap().trim();

            // very light validation
            if asof.is_empty() || account.is_empty() || sec.is_empty() || qty.is_empty() {
                return Err(bad_req("missing required fields"));
            }

            // Push row
            writer.write(&[
                &asof.parse::<chrono::NaiveDate>().map_err(|_| bad_req("bad asof_date"))? as &dyn ToSql,
                &account,
                &sec,
                &qty.parse::<f64>().map_err(|_| bad_req("bad qty"))?,
            ]).map_err(internal)?;
        }
        writer.finish().await.map_err(internal)?;
    }

    // Merge into target with UPSERT (exactly-once)
    tr.execute(
        "INSERT INTO account_positions (asof_date, account_id, sec_id, qty)
         SELECT asof_date, account_id, sec_id, qty
         FROM t_upload_accounts
         ON CONFLICT (asof_date, account_id, sec_id)
         DO UPDATE SET qty = EXCLUDED.qty;", &[]
    ).await.map_err(internal)?;

    tr.commit().await.map_err(internal)?;

    Ok((StatusCode::ACCEPTED, Json(json!({"status":"accepted"}))))
}

// Helpers

use tokio_postgres::types::Type::{Date, Text, Float8};
use axum::http::StatusCode as SC;

fn map_header(header: &StringRecord, required: &[&str]) -> Option<std::collections::HashMap<&'static str, usize>> {
    use std::collections::HashMap;
    let mut idx = HashMap::new();
    for (i, h) in header.iter().enumerate() {
        let k = h.trim().to_ascii_lowercase();
        match k.as_str() {
            "asof_date" => { idx.insert("asof_date", i); }
            "account_id" => { idx.insert("account_id", i); }
            "sec_id" => { idx.insert("sec_id", i); }
            "qty" => { idx.insert("qty", i); }
            _ => {}
        }
    }
    if required.iter().all(|k| idx.contains_key(k)) { Some(idx) } else { None }
}

fn bad_req(msg: &str) -> (SC, String) { (SC::BAD_REQUEST, msg.to_string()) }
fn internal<E: std::fmt::Display>(e: E) -> (SC, String) { (SC::INTERNAL_SERVER_ERROR, e.to_string()) }
