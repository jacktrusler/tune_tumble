use exitfailure::ExitFailure;
use rand::Rng;
use reqwest::header::USER_AGENT;
use reqwest::Url;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SongInfo {
    count: i32,
    recordings: Vec<Title>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Title {
    title: String,
    artist_credit: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Artist {
    name: String,
}

impl SongInfo {
    async fn get(query: &String) -> Result<Self, ExitFailure> {
        let url = format!(
            "https://musicbrainz.org/ws/2/recording?query={}&fmt=json",
            query,
        );

        let client = reqwest::Client::new();
        let url = Url::parse(&*url)?;
        let res = client
            .get(url)
            .header(USER_AGENT, "Random Songinator")
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }

    //Chat GPT added this D:
    async fn play(&self, index: usize) -> Result<(), ExitFailure> {
        let title = &self.recordings[index].title;
        let artist = &self.recordings[index].artist_credit[0].name;
        let query = format!("{} {}", title, artist);
        let query = query.replace(" ", "+");
        let url = format!("https://www.youtube.com/results?search_query={}", query);

        // Open the search results in the default web browser
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", "start", &url])
                .spawn()
                .expect("failed to start browser");
        } else if cfg!(target_os = "macos") {
            Command::new("open")
                .arg(&url)
                .spawn()
                .expect("failed to start browser");
        } else if cfg!(target_os = "linux") {
            Command::new("xdg-open")
                .arg(&url)
                .spawn()
                .expect("failed to start browser");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let mut args: Vec<String> = env::args().collect();
    args.drain(0..1);
    let query: String;

    if args.len() == 0 {
        println!("Specify a search query");
        return Ok(());
    } else if args.len() > 1 {
        query = args.join(" ");
        println!("{:?}", query);
    } else {
        query = args[0].clone();
        println!("{:?}", query);
    }

    let mut sp = Spinner::new(Spinners::Arrow3, "Searching for song".to_owned());

    let res = SongInfo::get(&query).await?;
    let res2 = res.clone();

    sp.stop();
    println!("");
    if res.recordings.len() == 0 {
        println!("This query returned no results.");
        return Ok(());
    }

    let all_songs: Vec<String> = res
        .recordings
        .into_iter()
        .map(|song| {
            format!(
                "Song: {}   |   Artist: {}",
                song.title, song.artist_credit[0].name
            )
        })
        .collect();
    let mut rng = rand::thread_rng();
    let random: usize = rng.gen_range(0..all_songs.len());
    println!("{}", all_songs[random]);
    res2.play(random).await?;

    Ok(())
}
