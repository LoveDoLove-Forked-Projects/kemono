use std::time::Duration;

use anyhow::Result;
use nyquest::{r#async::Request, AsyncClient, ClientBuilder, Method};
use url::Url;

use crate::model::{post_info::PostInfo, posts::Post, user_profile::UserProfile};

#[derive(Clone, Debug)]
pub struct API {
    client: AsyncClient,
    base_url: Url,
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 GLS/100.10.9939.100";

impl API {
    pub async fn try_new() -> Result<Self> {
        Self::try_with_base_url("https://kemono.cr").await
    }

    pub async fn try_with_base_url(base_url: impl AsRef<str>) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())?;
        Ok(API {
            client: ClientBuilder::default()
                .base_url(base_url.as_ref())
                .user_agent(USER_AGENT)
                .with_header(http::header::ACCEPT.as_str(), "text/css")
                .with_header(http::header::REFERER.as_str(), base_url.as_ref())
                .request_timeout(Duration::from_secs(10))
                .build_async()
                .await?,
            base_url,
        })
    }

    pub async fn head(&self, url: &str) -> Result<nyquest::r#async::Response> {
        let req = nyquest::Request::new(Method::custom("HEAD"), url.to_string());
        let resp = self.client.request(req).await?;
        Ok(resp)
    }

    pub async fn get_stream(
        &self,
        url: &str,
        start_pos: u64,
    ) -> Result<nyquest::r#async::Response> {
        let req = nyquest::Request::get(url.to_string())
            .with_header(http::header::RANGE.as_str(), format!("bytes={start_pos}-"));
        Ok(self.client.request(req).await?)
    }

    pub async fn get_posts(
        &self,
        web_name: &str,
        user_id: &str,
        offset: usize,
    ) -> Result<Vec<Post>> {
        let url = format!("/api/v1/{web_name}/user/{user_id}/posts?o={offset}",);
        let req = nyquest::Request::get(url.to_string());

        let resp = self.client.request(req).await?;
        if !resp.status().is_successful() {
            let status = resp.status();
            return Err(anyhow::anyhow!("GET {url} failed with status {status}",));
        }
        let val = resp.json().await?;
        Ok(val)
    }

    pub async fn get_post_info(
        &self,
        web_name: &str,
        user_id: &str,
        post_id: &str,
    ) -> Result<PostInfo> {
        let base_url = &self.base_url;
        let url = format!("/api/v1/{web_name}/user/{user_id}/post/{post_id}");

        let req = Request::get(url.clone()).with_header(
            http::header::REFERER.as_str(),
            format!("{base_url}/{web_name}/user/{user_id}/post/{post_id}"),
        );
        let resp = self.client.request(req).await?;

        if !resp.status().is_successful() {
            let status = resp.status();
            return Err(anyhow::anyhow!("GET {url} failed with status {status}",));
        }
        let val = resp.json().await?;
        Ok(val)
    }

    pub async fn get_user_profile(&self, web_name: &str, user_id: &str) -> Result<UserProfile> {
        let base_url = &self.base_url;
        let url = format!("/api/v1/{web_name}/user/{user_id}/profile",);
        let req = Request::get(url.clone()).with_header(
            http::header::REFERER.as_str(),
            format!("{base_url}/{web_name}/user/{user_id}"),
        );

        let resp = self.client.request(req).await?;
        if !resp.status().is_successful() {
            let status = resp.status();
            return Err(anyhow::anyhow!("GET {url} failed with status {status}",));
        }
        let val = resp.json().await?;
        Ok(val)
    }
}
