#![feature(async_closure)]

use std::{fs, panic, thread};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use serenity::http::Http;
use serenity::model::prelude::Embed;
use lazy_static::{initialize, lazy_static};
use octocrab::models::repos::RepoCommit;
use octocrab::Octocrab;
use octocrab::Page;
use semver::Version;
use tokio::task;
use tokio::task::{block_in_place, spawn_blocking};

lazy_static! {
    pub static ref TOKEN: (u64, String) = {
    let file = fs::read_to_string("token.txt").unwrap();
    let mut split: Vec<&str> =  file.split("/").collect();
    let uid: u64 = split[5].parse().unwrap();
    (uid, split[6].to_owned())
    };
}


#[tokio::main]
async fn main() {
	initialize(&TOKEN);

	let octocrab = octocrab::instance();
	let mut highest = Version::new(0, 0, 0);
	let mut first = true;

	panic::set_hook(Box::new( |_| {
		thread::spawn(async ||{
			hook(Version::new(0,0,0), "Bot panicked lmao").await;
		});
	}));
	[0, 1, 2][5];

	loop {
		let commits = get_commits(octocrab.clone()).await;
		let commit = &commits.items[0];
		let mut v = Version::parse(&commit.commit.message.split_at(2).1).unwrap();
		if first {
			highest = v;
			first = false;
		} else {
			if v.major > highest.major || v.minor > v.minor {
				hook(v, &commit.html_url).await;
			}
		}


		sleep(Duration::from_secs(60));
	}
}


async fn hook(version: Version, url: &str) {
	let my_http_client = Http::new(&TOKEN.1);

	let webhook = my_http_client.get_webhook_with_token(TOKEN.0, &TOKEN.1).await.unwrap();

	let embed = Embed::fake(|e| {
		e.title("Updat!")
		// .color(Color::from_rgb(116, 16, 210))
		.description(version.to_string())
		// .thumbnail("https://avatars.githubusercontent.com/u/97326911?s=40&v=4")
		// .image(&content.img_url)
		.url(url)
		// .field("Want these news for your server too?", "https://news.wt.flareflo.dev", true)
		// .footer(|f| {
		//     f.icon_url("https://warthunder.com/i/favicons/mstile-70x70.png").text("Report bugs/issues: FlareFlo🦆#2800")
		// })
		// .timestamp(Timestamp::now())
	});

	webhook.execute(my_http_client, false, |w| {
		w.content(&format!("[{}]()", url));
		w.embeds(vec![embed]);
		w
	}).await.unwrap();
}

async fn get_commits(octocrab: Arc<Octocrab>) -> Page<RepoCommit> {
	octocrab.repos("gszabi99", "War-Thunder-Datamine")
			.list_commits()
			.branch("master")
			.per_page(10)
			.send().await.unwrap()
}