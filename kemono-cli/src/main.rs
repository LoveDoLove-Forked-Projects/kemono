use std::{fs, io::IsTerminal, path::PathBuf, sync::atomic::Ordering};

use anyhow::Result;
use argh::FromArgs;
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use kemono_cli::{
    helper::{batch::download_all, ctx::Args, single::download_one},
    utils::{extract_info, DownloadInfo},
    DONE,
};

#[derive(FromArgs, Debug)]
#[argh(description = "Download tool")]
struct Cli {
    /// kemono URL to fetch posts, can be user profile or single post
    ///
    /// Example:
    ///
    /// https://kemono.su/fanbox/user/4107959
    ///
    /// https://kemono.su/fanbox/user/4107959/post/7999699
    #[argh(positional)]
    url: String,

    /// output directory of fetched posts
    #[argh(option, default = "PathBuf::from(\"./download\")")]
    output_dir: PathBuf,

    /// maximium number of tasks running in background concurrently
    #[argh(option, short = 'p', default = "4")]
    max_concurrency: usize,

    /// whitelist regex for title.
    ///
    /// specify multiple times means 'AND' semantic
    #[argh(option, short = 'w')]
    whitelist_regex: Vec<String>,

    /// blacklist regex for title.
    ///
    /// specify multiple times means 'AND' semantic
    #[argh(option, short = 'b')]
    blacklist_regex: Vec<String>,

    /// whitelist regex for filename.
    ///
    /// specify multiple times means 'AND' semantic
    #[argh(option, short = 'W')]
    whitelist_filename_regex: Vec<String>,

    /// blacklist regex for filename.
    ///
    /// specify multiple times means 'AND' semantic
    #[argh(option, short = 'B')]
    blacklist_filename_regex: Vec<String>,

    /// only fetch videos since this date. format like: 2025-01-01
    #[argh(option)]
    start_date: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    kdam::term::init(std::io::stderr().is_terminal());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true)
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .from_env_lossy(),
                ),
        )
        .init();

    let cli = argh::from_env();
    info!("Started with arguments: {cli:?}");
    let Cli {
        url,
        output_dir,
        max_concurrency,
        whitelist_regex,
        blacklist_regex,
        whitelist_filename_regex,
        blacklist_filename_regex,
        start_date,
    } = cli;

    info!("Download URL: {}", &url);

    fs::create_dir_all(&output_dir)?;

    ctrlc::set_handler(move || {
        if DONE.load(Ordering::Acquire) {
            info!("Signal handler called twice, force-exiting");
            std::process::exit(127);
        } else {
            info!("Signal handler called");
        }
        DONE.store(true, Ordering::Release);
    })?;

    let DownloadInfo {
        api_base_url,
        web_name,
        user_id,
        post_id,
    } = extract_info(&url)?;

    let args = Args::builder()
        .web_name(web_name)
        .user_id(user_id)
        .max_concurrency(max_concurrency)
        .output_dir(output_dir)
        .whitelist_regexes(whitelist_regex)
        .blacklist_regexes(blacklist_regex)
        .whitelist_filename_regexes(whitelist_filename_regex)
        .blacklist_filename_regexes(blacklist_filename_regex)
        .api_base_url(api_base_url)
        .start_date(
            start_date.and_then(|date| chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()),
        )
        .build()?;

    info!("args: {args:#?}");

    match post_id {
        Some(post_id) => {
            if let Err(e) = download_one(&args, &post_id).await {
                error!("{e}");
            }
        }
        None => {
            if let Err(e) = download_all(&args).await {
                error!("{e}");
            }
        }
    }

    info!("Task Exit");

    Ok(())
}
