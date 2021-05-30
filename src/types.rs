//! Item types returned by the API.

use crate::client::{HnClient, ItemLoader};
use crate::result::Result;
use async_graphql::{dataloader::DataLoader, ComplexObject, Context, Interface, SimpleObject};
use serde::Deserialize;

/// An API item, for example a story or a comment.
#[derive(Debug, Clone, Deserialize, Interface)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
#[graphql(
    field(name = "id", type = "&u32"),
    field(name = "title", type = "Option<&str>"),
    field(name = "author", type = "Option<&str>")
)]
pub enum Item {
    /// A story.
    Story(Story),
    /// A comment.
    Comment(Comment),
    /// A job.
    Job(Job),
    /// A poll.
    Poll(Poll),
    /// A poll option belonging to a poll.
    Pollopt(Pollopt),
}

/// A story.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Story {
    /// The item's unique id.
    pub id: u32,
    /// The total comment count.
    pub descendants: u32,
    /// The username of the item's author.
    pub by: String,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// The story's score.
    pub score: u32,
    #[graphql(skip)]
    /// The title of the story.
    pub title: String,
    /// The URL of the story.
    pub url: Option<String>,
    /// The story text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

#[ComplexObject]
impl Story {
    async fn title(&self) -> Option<&str> {
        Some(&self.title)
    }

    async fn author(&self) -> Option<&str> {
        Some(&self.by)
    }

    async fn kids_connection(&self, ctx: &Context<'_>, limit: Option<usize>) -> Result<Vec<Item>> {
        let limit = limit.unwrap_or_default();
        let kids = self
            .kids
            .clone()
            .unwrap_or_default()
            .into_iter()
            .take(limit)
            .collect::<Vec<_>>();

        Ok(ctx
            .data_unchecked::<DataLoader<ItemLoader>>()
            .load_many(kids)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, v)| v)
            .collect())
    }
}

/// A comment.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Comment {
    /// The item's unique id.
    pub id: u32,
    /// The username of the item's author.
    pub by: String,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// The comment's parent: either another comment or the relevant story.
    pub parent: u32,
    /// The comment text. HTML.
    pub text: String,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

#[ComplexObject]
impl Comment {
    async fn title(&self) -> Option<&str> {
        None
    }
    async fn author(&self) -> Option<&str> {
        Some(&self.by)
    }
}

/// A job.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Job {
    /// The item's unique id.
    pub id: u32,
    /// The story's score, or the votes for a pollopt.
    pub score: u32,
    /// The job text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
    #[graphql(skip)]
    /// The title of the job.
    pub title: String,
    /// The URL of the story.
    pub url: Option<String>,
}

#[ComplexObject]
impl Job {
    async fn title(&self) -> Option<&str> {
        Some(&self.title)
    }
    async fn author(&self) -> Option<&str> {
        None
    }
}

/// A poll.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Poll {
    /// The item's unique id.
    pub id: u32,
    /// The username of the item's author.
    pub by: String,
    /// The total comment count.
    pub descendants: u32,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// A list of related pollopts, in display order.
    pub parts: Option<Vec<u32>>,
    /// The story's score.
    pub score: u32,
    #[graphql(skip)]
    /// The title of the story.
    pub title: String,
    /// The story text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

#[ComplexObject]
impl Poll {
    async fn title(&self) -> Option<&str> {
        Some(&self.title)
    }
    async fn author(&self) -> Option<&str> {
        Some(&self.by)
    }
}

/// A poll option belonging to a poll.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Pollopt {
    /// The item's unique id.
    pub id: u32,
    /// The username of the item's author.
    pub by: String,
    /// The pollopt's associated poll.
    pub poll: u32,
    /// The votes for a pollopt.
    pub score: u32,
    /// The story text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

#[ComplexObject]
impl Pollopt {
    async fn title(&self) -> Option<&str> {
        None
    }
    async fn author(&self) -> Option<&str> {
        Some(&self.by)
    }
}

/// A user profile.
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    /// The user's unique username. Case-sensitive.
    pub id: String,
    /// Creation date of the user, in Unix Time.
    pub created: u64,
    /// The user's karma.
    pub karma: u32,
    /// Delay in minutes between a comment's creation and its visibility to
    /// other users.
    pub delay: Option<u32>,
    /// The user's optional self-description. HTML.
    pub about: Option<String>,
    /// List of the user's stories, polls and comments.
    pub submitted: Vec<u32>,
}

/// A list of recently updated items and users.
#[derive(Debug, Clone, Deserialize)]
pub struct Updates {
    /// A list of recently changed items.
    pub items: Vec<u32>,
    /// A list of recently changed usernames.
    pub profiles: Vec<String>,
}
