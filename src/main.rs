use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{
    dataloader::DataLoader, Context, EmptyMutation, EmptySubscription, Object, Schema,
};
use async_graphql_warp::{BadRequest, Response};
use futures::future::join_all;
use http::StatusCode;
use std::convert::Infallible;
use warp::{http::Response as HttpResponse, Filter, Rejection};

mod client;
mod result;
mod types;
use client::{HnClient, ItemLoader};
use result::Result;
use types::*;

#[tokio::main]
async fn main() {
    let client = HnClient::init().unwrap();

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(client.clone())
        .data(DataLoader::new(ItemLoader { client }))
        .finish();

    println!("Playground: http://localhost:8000");

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (
            Schema<Query, EmptyMutation, EmptySubscription>,
            async_graphql::Request,
        )| async move { Ok::<_, Infallible>(Response::from(schema.execute(request).await)) },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    });

    let routes = graphql_playground
        .or(graphql_post)
        .recover(|err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        });

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}

struct Query;

#[Object]
impl Query {
    async fn top(&self, ctx: &Context<'_>, limit: Option<usize>) -> Result<Vec<Item>> {
        let client = ctx.data_unchecked::<HnClient>();
        let limit = limit.unwrap_or(10);
        let ids = client
            .get_top_stories()
            .await?
            .into_iter()
            .take(limit)
            .collect::<Vec<_>>();

        Ok(ctx
            .data_unchecked::<DataLoader<ItemLoader>>()
            .load_many(ids)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, v)| v)
            .collect())
    }
}
