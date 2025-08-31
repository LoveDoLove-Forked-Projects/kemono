use std::sync::atomic::Ordering;

use anyhow::{anyhow, Result};

use kemono_api::model::posts::Post;
use kemono_api::API;
use tracing::{debug, error};

use crate::helper::post;
use crate::utils::normalize_pathname;
use crate::DONE;

use crate::helper::ctx;
use crate::helper::utils::get_author_name;

pub async fn download_all(ctx: impl ctx::Context<'_>) -> Result<()> {
    let web_name = ctx.web_name();
    let user_id = ctx.user_id();
    let base_url = ctx.api_base_url();

    let api = API::try_with_base_url(base_url).await?;
    let mut offset = 0;

    loop {
        if DONE.load(Ordering::Acquire) {
            break;
        }

        let posts = api
            .get_posts(web_name, user_id, offset)
            .await
            .map_err(|e| anyhow!("failed to fetch props: {e}"))?;

        let length = posts.len();
        if length == 0 {
            break;
        }
        debug!("fetched {length} posts");

        let author = get_author_name(&api, web_name, user_id).await?;
        let author = normalize_pathname(&author);

        for Post {
            id: ref post_id,
            title: ref post_title,
            ..
        } in posts
        {
            if DONE.load(Ordering::Acquire) {
                error!("Received SIGINT, exiting");
                break;
            }
            post::download_post(&ctx, &api, &post_id, &post_title, &author).await?;
        }

        offset += length;
    }

    Ok(())
}
