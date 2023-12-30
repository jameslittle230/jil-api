#![allow(dead_code)]

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum CachedIpInfoResponseType {
    CacheHit,
    CacheMiss,
}

pub(crate) struct CachedIpInfoResponse {
    pub(crate) ip_info: IpInfo,
    response_type: CachedIpInfoResponseType,
}

impl CachedIpInfoResponse {
    fn new(ip_info: IpInfo, response_type: CachedIpInfoResponseType) -> Self {
        Self {
            ip_info,
            response_type,
        }
    }
}

#[derive(Debug)]
pub(crate) struct CachedIpInfoClient {
    client: IpInfoApiClient,
    cache: HashMap<String, IpInfo>,
}

impl CachedIpInfoClient {
    pub(crate) fn new(token: String) -> Self {
        Self {
            client: IpInfoApiClient::new(token),
            cache: HashMap::new(),
        }
    }

    pub(crate) async fn get_ip_info(&mut self, ip: &str) -> Result<CachedIpInfoResponse> {
        if let Some(ip_info) = self.cache.get(ip) {
            return Ok(CachedIpInfoResponse::new(
                ip_info.clone(),
                CachedIpInfoResponseType::CacheHit,
            ));
        }

        let ip_info = self.client.get_ip_info(ip).await?;
        self.cache.insert(ip.to_string(), ip_info.clone());

        Ok(CachedIpInfoResponse::new(
            ip_info,
            CachedIpInfoResponseType::CacheMiss,
        ))
    }
}

#[derive(Debug)]
struct IpInfoApiClient {
    reqwest_client: reqwest::Client,
    token: String,
}

impl IpInfoApiClient {
    fn new(token: String) -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            token,
        }
    }

    async fn get_ip_info(&self, ip: &str) -> Result<IpInfo> {
        let url = format!("https://ipinfo.io/{}?token={}", ip, self.token);
        let response = self.reqwest_client.get(&url).send().await?;
        let text = response.text().await?;
        let ip_info = serde_json::from_str::<IpInfo>(&text)?;
        Ok(ip_info)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct IpInfo {
    ip: String,
    city: String,
    region: String,
    country: String,
    loc: String,
    org: String,
    postal: String,
    timezone: String,
}

impl IpInfo {
    pub(crate) fn loc_to_string(&self) -> String {
        format!("{}, {}, {}", self.city, self.region, self.country)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        dotenv::dotenv().ok();
        let client = IpInfoApiClient::new(std::env::var("IPINFO_KEY").unwrap());
        let ip_info = client.get_ip_info("73.253.33.205").await.unwrap();
        assert_eq!(ip_info.ip, "73.253.33.205");
    }

    #[tokio::test]
    async fn it_works_with_cache() {
        dotenv::dotenv().ok();
        let mut client = CachedIpInfoClient::new(std::env::var("IPINFO_KEY").unwrap());
        let ip_info = client.get_ip_info("73.253.33.205").await.unwrap();
        assert_eq!(ip_info.ip_info.ip, "73.253.33.205");
        assert_eq!(ip_info.response_type, CachedIpInfoResponseType::CacheMiss);

        let ip_info = client.get_ip_info("73.253.33.205").await.unwrap();
        assert_eq!(ip_info.ip_info.ip, "73.253.33.205");
        assert_eq!(ip_info.response_type, CachedIpInfoResponseType::CacheHit);
    }

    #[tokio::test]
    async fn error_response() {
        dotenv::dotenv().ok();
        let mut client = CachedIpInfoClient::new(std::env::var("IPINFO_KEY").unwrap());
        let ip_info = client.get_ip_info("not an ip").await;
        assert!(ip_info.is_err());
    }
}
