extern crate reqwest;
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::StatusCode;
use std::collections::HashMap;
use std::str::FromStr;
use anyhow::Error;
use log::{error, info};
use crate::ipc::ModelMutateResultData;
use crate::model::{SettingController, Meeting};
use chrono::{Local, DateTime, NaiveDateTime, Utc};
const ENDPOINT_BASE: &str = "https://api.affinity.co/";
use serde_json::json;

fn get_api_key() -> Result<String, Error> {
    let setting = SettingController::new(crate::model::SettingPath::Default).get_setting().unwrap();
    if setting.affinity_api_token.is_empty() {
        return Err(anyhow::Error::msg("Affinity API token is not set"));
    } else {
        return Ok(setting.affinity_api_token);
    }
}

fn build_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers
}

fn check_err(r: &reqwest::blocking::Response) -> Result<(), Error> {
    if r.status() == StatusCode::OK {
        return Ok(());
    } else {
        error!("Request Error - {}", r.status());
        return Err(anyhow::Error::msg("Error occured during request"));
    } 
}

fn check_async_err(r: &reqwest::Response) -> Result<(), Error> {
    if r.status() == StatusCode::OK {
        return Ok(());
    } else {
        return Err(anyhow::Error::msg("Error occured during request"));
    } 
}

fn build_request(
    client: &Client,
    api_key: &str,
    method: reqwest::Method,
    call: &str,
) -> RequestBuilder {
    client
        .request(method, call)
        .basic_auth("", Some(api_key))
        .headers(build_headers())
}

fn build_async_request(
    client: &reqwest::Client,
    api_key: &str,
    method: reqwest::Method,
    call: &str,
) -> reqwest::RequestBuilder {
    client
        .request(method, call)
        .basic_auth("", Some(api_key))
        .headers(build_headers())
}


fn get(client: &Client, api_key: &str, call: &str) -> Result<serde_json::Value, Error> {
    let request = build_request(client, api_key, reqwest::Method::GET, call);
    match request.send() {
        Ok(response) => {
            match check_err(&response) {
                Ok(_) => {
                    match response.json() {
                        Ok(json) => Ok(json),
                        Err(error) => Err(error.into()),
                    }
                },
                Err(error) => Err(error),
            }
        }
        Err(error) => return Err(error.into()),
    }
}


async fn async_get(client: &reqwest::Client, api_key: &str, call: &str) -> Result<serde_json::Value, Error> {
    let request = build_async_request(client, api_key, reqwest::Method::GET, call);
    match request.send().await {
        Ok(response) => {
            match check_async_err(&response) {
                Ok(_) => {
                    match response.json().await {
                        Ok(json) => Ok(json),
                        Err(error) => Err(error.into()),
                    }
                },
                Err(error) => Err(error),
            }
        }
        Err(error) => return Err(error.into()),
    }
}


fn post(
    client: &Client,
    api_key: &str,
    call: &str,
    data: HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, Error> {
    let request = build_request(client, api_key, reqwest::Method::POST, call).json(&data);
    info!("POST: {:?}", request);
    match request.send() {
        Ok(response) => {
            match check_err(&response) {
                Ok(_) => {
                    info!("post - Response: {:?}", response);
                    match response.json() {
                        Ok(json) => Ok(json),
                        Err(error) => Err(error.into())
                    }
                },
                Err(error) => Err(error)
            }
        }
        Err(error) => Err(error.into())
    }
}

fn _put(
    client: &Client,
    api_key: &str,
    call: &str,
    data: HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, Error> {
    let request = build_request(client, api_key, reqwest::Method::PUT, call).json(&data);
    match request.send() {
        Ok(response) => {
            match check_err(&response) {
                Ok(_) => {
                    match response.json() {
                        Ok(json) => Ok(json),
                        Err(error) => Err(error.into()),
                    }
                },
                Err(error) => Err(error.into()),
            }
        }
        Err(error) => Err(error.into()),
    }
}

fn _delete(client: &Client, api_key: &str, call: &str) -> Result<serde_json::Value, Error> {
    let request = build_request(client, api_key, reqwest::Method::DELETE, call);
    match request.send() {
        Ok(response) => {
            match check_err(&response) {
                Ok(_) => {
                    match response.json() {
                        Ok(json) => Ok(json),
                        Err(error) => Err(error.into()),
                    }
                },
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error.into()),
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub fn get_lists() -> Result<serde_json::Value, Error> {
    let client = Client::new();
    let api_key = get_api_key()?;
    let call = format!("{}/lists", ENDPOINT_BASE);
    let result = get(&client, &api_key, &call);
    match result {
        Ok(json) => {
            return Ok(json);
        },
        Err(error) => {
            error!("get_all_lists: {:?}", error);
            return Err(error);
        },
    }
}

fn get_list_entries(list_id: &str) -> Result<serde_json::Value, Error> {
    let client = Client::new();
    let api_key = get_api_key()?;
    let call = format!("{}/lists/{}", ENDPOINT_BASE, list_id);
    let result = get(&client, &api_key, &call);
    match result {
        Ok(json) => {
            return Ok(json);
        },
        Err(error) => {
            error!("get_all_lists: {:?}", error);
            return Err(error);
        },
    }
}

fn post_add_org_to_list(list_id: &str, org_id: &str) -> Result<serde_json::Value, Error> {
    let client = Client::new();
    let api_key = get_api_key()?;
    let call = format!("{}/lists/{}/list-entries", ENDPOINT_BASE, list_id);
    let mut data = HashMap::new();
    data.insert("entity_id".to_string(), serde_json::Value::Number(serde_json::Number::from_str(org_id)?));
    let result = post(&client, &api_key, &call, data);
    match result {
        Ok(json) => {
            return Ok(json);
        },
        Err(error) => {
            error!("post_add_org_to_list: {:?}", error);
            return Err(error);
        },
    }
}

pub fn add_to_list(meeting: Meeting) -> Result<(), Error> {
    // Verif there is a org_id
    if meeting.company_id.is_empty() {
        return Err(anyhow::Error::msg("Affinity organization id is needed for this action"));
    }
    let setting = SettingController::new(crate::model::SettingPath::Default).get_setting().unwrap();
    match setting.affinity_crm_list_id {
        Some(list_id) => {
            // Get list entries
            let entries = &get_list_entries(list_id.as_str())?["list_entries"];
            print!("Entries: {:?}", entries);
            let entry_alread_exist = match entries.as_array() {
                Some(entries) => entries.iter().any(|entry| entry["entity_id"] == meeting.company_id),
                None => false,
            };
            if !entry_alread_exist {
                let result = post_add_org_to_list(list_id.as_str(), meeting.company_id.as_str())?;
                println!("{}", result)
            }
            return Ok(());
        },
        None => {
            return Err(anyhow::Error::msg("Affinity CRM list id is not set | Features ignored"));
        },
    }
}

pub fn get_persons(
    term: &str,
    page_size: Option<&str>,
    page_token: Option<&str>,
) -> Result<serde_json::Value, Error> {
    let client = Client::new();
    let api_key = get_api_key()?;

    let mut call = format!("{}/persons", ENDPOINT_BASE);
    if term.is_empty() {
        return Err(anyhow::Error::msg("Term for the search is needed"));
    }
    call += &format!("?term={}", term);
    if let Some(size) = page_size {
        call += &format!("&page_size={}", size);
    }
    if let Some(token) = page_token {
        call += &format!("&page_token={}", token);
    }
    let result = get(&client, &api_key, &call);
    match result {
        Ok(json) => {
            info!("get_persons: {:?}", json);
            return Ok(json);
        },
        Err(error) => {
            error!("get_persons: {:?}", error);
            return Err(error);
        },
    }
}

pub fn get_organizations(
    org_id: &str,
) -> Result<serde_json::Value, Error> {
    let client = Client::new();
    let api_key = get_api_key()?;
    if org_id.is_empty() {
        return Err(anyhow::Error::msg("organization id is needed"))
    }
    let call = format!("{}/organizations/{}", ENDPOINT_BASE, org_id);
    let result = get(&client, &api_key, &call);
    match result {
        Ok(json) => {
            return Ok(json);
        },
        Err(error) => {
            error!("[Affinity] get_organizations: {:?}", error);
            return Err(error);
        },        
    }
}

pub fn _search_organizations(
    term: &str,
    page_size: Option<&str>,
    page_token: Option<&str>,
) -> Result<serde_json::Value, Error> {
    let client = Client::new();
    let api_key = get_api_key()?;

    let mut call = format!("{}/organizations", ENDPOINT_BASE);
    if term.is_empty() {
        return Err(anyhow::Error::msg("Term for the search is needed"))
    }
    call += &format!("?term={}", term);
    if let Some(size) = page_size {
        call += &format!("&page_size={}", size);
    }
    if let Some(token) = page_token {
        call += &format!("&page_token={}", token);
    }
    let result = get(&client, &api_key, &call);
    match result {
        Ok(json) => {
            return Ok(json);
        },
        Err(error) => {
            error!("[Affinity] search_organizations: {:?}", error);
            return Err(error);
        },
    }
}


pub async fn async_search_organizations(
    term: &str,
    page_size: Option<&str>,
    page_token: Option<&str>,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let api_key = get_api_key()?;

    let mut call = format!("{}/organizations", ENDPOINT_BASE);
    if term.is_empty() {
        return Err(anyhow::Error::msg("Term for the search is needed"))
    }
    call += &format!("?term={}", term);
    if let Some(size) = page_size {
        call += &format!("&page_size={}", size);
    }
    if let Some(token) = page_token {
        call += &format!("&page_token={}", token);
    }
    let result = async_get(&client, &api_key, &call).await;
    match result {
        Ok(json) => {
            return Ok(json);
        },
        Err(error) => {
            error!("[Affinity] search_organizations: {:?}", error);
            return Err(error);
        },
    }
}


pub fn create_note(
    meeting: &Meeting,
) -> Result<ModelMutateResultData, Error> {
    // Verif there is a org_id
    if meeting.company_id.is_empty() {
        return Err(anyhow::Error::msg("Please specify an company"));
    }
    let id = meeting.get_uuid();
    let client = Client::new();
    let api_key = get_api_key()?;

    let call = format!("{}/notes", ENDPOINT_BASE);

    // Convert date to local time
    let utc = NaiveDateTime::parse_from_str(meeting.datetime.as_str(), "%Y-%m-%dT%H:%M:%SZ")
        .map(|ndt| DateTime::<Utc>::from_utc(ndt, Utc));
    let date = match utc {
        Ok(utc) => {
            let date_with_timezone = utc.with_timezone(&Local).to_string();
            date_with_timezone.split(" -").next().unwrap_or(&date_with_timezone).to_string()
        },
        Err(error) => {
            error!("{}", format!("Error converting date {} - {}", meeting.datetime.to_string(), error.to_string()));
            meeting.datetime.to_string()
        }
    };

    let mut note = format!("<p>{}<p><p>Generated with Watson</p><p>{}<p>{}", meeting.title, date, meeting.summary);
    let mut data = HashMap::new();

    match meeting.publish_with_note {
        Some(true) => {
            note = format!("{}<br><p><b>Personal note</b></p>{}", note, meeting.note);
        }
        _ => (),
    }

    data.insert("organization_ids".to_string(), serde_json::Value::Array([serde_json::Value::String(meeting.company_id.clone())].to_vec()));
    data.insert("content".to_string(), serde_json::Value::String(note));
    data.insert("type".to_string(), json!(2));
    let result = post(&client, &api_key, &call, data);
    match result {
        Ok(_) => (),
        Err(error) => error!("[Affinity] Error while publishing notes: {:?}", error),
    }
    return Ok(ModelMutateResultData { id } );
}
