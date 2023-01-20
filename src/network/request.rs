use crate::network::Credential;
use std::{collections::HashMap, str::FromStr};

pub struct Request {}

impl Request {
    pub async fn send(
        method: &str,
        url: &str,
        params: Option<&HashMap<String, String>>,
        data: Option<&mut HashMap<String, String>>,
        credential: Option<&Credential>,
        no_csrf: bool,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let method = method.to_uppercase();
        let client = reqwest::Client::new();

        // From hashmap to a single string
        let cookies = if let Some(cert) = credential {
            cert.get_cookies()
                .unwrap()
                .iter()
                .map(|(k, v)| format!("{}={}", k, v).replace(";", "%3B"))
                .collect::<Vec<_>>()
                .join(";")
        } else {
            "".to_owned()
        };

        // Create a request builder
        let mut req_builder = client
            .request(
                reqwest::Method::from_str(&method).unwrap(),
                reqwest::Url::from_str(url).unwrap(),
            )
            .header("Referer", "https://www.bilibili.com");

        if params.is_some() {
            todo!()
        }

        // Add cookies into headers of request
        if cookies.len() > 0 {
            req_builder = req_builder.header("Cookie", cookies);
        }

        if let Some(hm) = data {
            // Add csrf field into form
            if !no_csrf && credential.is_some() {
                hm.insert("csrf".to_owned(), credential.unwrap().bili_jct.clone());
                hm.insert(
                    "csrf_token".to_owned(),
                    credential.unwrap().bili_jct.clone(),
                );
                req_builder = req_builder.form(hm);
            }
        }

        let resp = req_builder.send().await?;

        Ok(resp)
    }
}
