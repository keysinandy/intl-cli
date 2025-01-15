use chrono::{DateTime, TimeZone, Utc};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::{to_string, Map, Value};

use crate::utils::hash::{get_hash, sha256};

use super::translate::{Payload, Translate};

fn get_date(timestamp: i64) -> String {
    let dt: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    dt.format("%Y-%m-%d").to_string()
}

#[derive(Deserialize, Debug)]
pub struct ResponseError {
    pub Code: String,
    pub Message: String,
}
#[derive(Deserialize, Debug)]
pub struct Response {
    pub RequestId: String,
    pub Source: String,
    pub Target: String,
    pub TargetTextList: Vec<String>,
    pub UsedAmount: u32,
    pub Error: Option<ResponseError>,
}

#[derive(Deserialize, Debug)]
pub struct RequestResponse {
    pub Response: Response,
}
#[tokio::main]
pub async fn generate_by_tencent<T: Payload>(
    translate: &Translate<T>,
    pair_list: &Vec<(String, Value)>,
    secret_id: &str,
    secret_key: &str,
) -> Result<RequestResponse, Box<dyn std::error::Error>> {
    const TOKEN: &str = "";
    const HOST: &str = "tmt.tencentcloudapi.com";
    const SERVICE: &str = "tmt";
    const REGION: &str = "ap-shanghai";
    const ACTION: &str = "TextTranslateBatch";
    const VERSION: &str = "2018-03-21";

    let timestamp = Utc::now().timestamp();
    let date = get_date(timestamp);

    let payload = translate.payload.to_string(pair_list);

    // Step 1: Build canonical request
    let signed_headers = "content-type;host";
    let hashed_request_payload = get_hash(payload.as_bytes());
    let http_request_method = "POST";
    let canonical_uri = "/";
    let canonical_query_string = "";
    let canonical_headers = format!(
        "content-type:application/json; charset=utf-8\nhost:{}\n",
        HOST
    );

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        http_request_method,
        canonical_uri,
        canonical_query_string,
        canonical_headers,
        signed_headers,
        hashed_request_payload
    );

    // Step 2: Build string to sign
    let algorithm = "TC3-HMAC-SHA256";
    let hashed_canonical_request = get_hash(canonical_request.as_bytes());
    let credential_scope = format!("{}/{}/tc3_request", date, SERVICE);
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, timestamp, credential_scope, hashed_canonical_request
    );

    // Step 3: Calculate signature
    let k_date = sha256(date.as_bytes(), format!("TC3{}", secret_key).as_bytes());
    let k_service = sha256(SERVICE.as_bytes(), &k_date);
    let k_signing = sha256(b"tc3_request", &k_service);
    let signature = hex::encode(sha256(string_to_sign.as_bytes(), &k_signing));

    // Step 4: Build Authorization
    let authorization = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, secret_id, credential_scope, signed_headers, signature
    );

    // Step 5: Create and send request
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(&authorization)?);
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert("Host", HeaderValue::from_str(HOST)?);
    headers.insert("X-TC-Action", HeaderValue::from_str(ACTION)?);
    headers.insert(
        "X-TC-Timestamp",
        HeaderValue::from_str(&timestamp.to_string())?,
    );
    headers.insert("X-TC-Version", HeaderValue::from_str(VERSION)?);

    if !REGION.is_empty() {
        headers.insert("X-TC-Region", HeaderValue::from_str(REGION)?);
    }

    if !TOKEN.is_empty() {
        headers.insert("X-TC-Token", HeaderValue::from_str(TOKEN)?);
    }

    let client = reqwest::Client::new();
    let response = client
        .post(format!("https://{}", HOST))
        .headers(headers)
        .body(payload)
        .send()
        .await?;

    let response = response.json::<RequestResponse>().await?;
    if let Some(e) = response.Response.Error {
        panic!("Request Result Error: {}", e.Message);
    }
    println!(
        "=========== Translate {:?} words, use amount {:?}===========",
        response.Response.TargetTextList.len(),
        response.Response.UsedAmount
    );

    return Ok(response);
}

pub struct TencentPayload {
    source: String,
    target: String,
    project_id: u32,
}

impl TencentPayload {
    pub fn new(source: String, target: String, project_id: u32) -> TencentPayload {
        TencentPayload {
            source,
            target,
            project_id,
        }
    }
}

impl Payload for TencentPayload {
    fn to_string(&self, pair_list: &Vec<(String, Value)>) -> String {
        let arr = pair_list.iter().map(|x| x.1.clone()).collect();
        let mut map = Map::new();
        map.insert("Source".to_string(), Value::String(self.source.to_string()));
        map.insert("Target".to_string(), Value::String(self.target.to_string()));
        map.insert(
            "ProjectId".to_string(),
            Value::Number(self.project_id.into()),
        );

        map.insert("SourceTextList".to_string(), Value::Array(arr));
        return to_string(&map).unwrap();
    }
    fn to_map(&self, pair_list: &Vec<(String, Value)>, list: Vec<String>) -> Map<String, Value> {
        let mut map = Map::new();
        let mut idx = 0;
        pair_list.iter().for_each(|x| {
            map.insert(x.0.to_string(), Value::String(list[idx].to_string()));
            idx += 1;
        });
        return map;
    }
}
