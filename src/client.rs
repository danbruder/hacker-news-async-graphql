use std::time::Duration;

use crate::result::Result;
use crate::types;
use async_graphql::dataloader::Loader;
use futures::future::{join_all, FutureExt};
use reqwest::{self, Client};
use std::collections::HashMap;

static API_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0";

/// The API client.
#[derive(Clone)]
pub struct HnClient {
    client: Client,
}

impl HnClient {
    /// Create a new `HnClient` instance.
    pub fn init() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self { client })
    }

    /// Return the item with the specified id.
    ///
    /// May return `None` if item id is invalid.
    pub async fn get_item(&self, id: u32) -> Result<Option<types::Item>> {
        Ok(self
            .client
            .get(&format!("{}/item/{}.json", API_BASE_URL, id))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return the user with the specified username.
    ///
    /// May return `None` if username is invalid.
    pub async fn get_user(&self, username: &str) -> Result<Option<types::User>> {
        Ok(self
            .client
            .get(&format!("{}/user/{}.json", API_BASE_URL, username))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return the id of the newest item.
    ///
    /// To get the 10 latest items, you can decrement the id 10 times.
    pub async fn get_max_item_id(&self) -> Result<u32> {
        Ok(self
            .client
            .get(&format!("{}/maxitem.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of top story item ids.
    pub async fn get_top_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/topstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of new story item ids.
    pub async fn get_new_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/newstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of best story item ids.
    pub async fn get_best_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/beststories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return up to 200 latest Ask HN story item ids.
    pub async fn get_ask_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/askstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return up to 200 latest Show HN story item ids.
    pub async fn get_show_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/showstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return up to 200 latest Job story item ids.
    pub async fn get_job_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/jobstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of items and users that have been updated recently.
    pub async fn get_updates(&self) -> Result<types::Updates> {
        Ok(self
            .client
            .get(&format!("{}/updates.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }
}

pub struct ItemLoader {
    pub client: HnClient,
}

#[async_trait::async_trait]
impl Loader<u32> for ItemLoader {
    type Value = types::Item;
    type Error = ();

    async fn load(&self, keys: &[u32]) -> std::result::Result<HashMap<u32, Self::Value>, ()> {
        let results = keys
            .into_iter()
            .map(|id| self.client.get_item(*id).map(move |res| (*id, res)))
            .collect::<Vec<_>>();

        Ok(join_all(results)
            .await
            .into_iter()
            .filter_map(|(id, res)| match res {
                Ok(Some(val)) => Some((id, val)),
                _ => None,
            })
            .collect())
    }
}
