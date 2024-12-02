use std::{
    fs::{self, File},
    io::{self},
    path::Path,
    time::Duration,
};

use bytes::Bytes;
use reqwest::blocking::{Client, Response};

use crate::filesystem::DayAndYear;

const REQUEST_TIMEOUT_SECS: u64 = 30;
const USER_AGENT: &str = "aoc-fs (https://github.com/guy_732/aoc-fs by guilhem.chaillou@gmail.com)";
const BASE_URL: &str = "https://adventofcode.com";

pub fn download_input(day: DayAndYear, save_path: &Path, session: &str) -> io::Result<()> {
    let url = format!("{BASE_URL}/{}/day/{}/input", day.year, day.day);
    let client = Client::new();
    let req = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Cookie", format!("session={}", session))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .expect("Could not build web request");

    let response = match client.execute(req) {
        Err(e) => {
            log::error!("Request failed for {:?}: {}", &url, e);
            return Err(io::Error::from_raw_os_error(libc::ENETDOWN));
        }

        Ok(response) => response,
    };

    let response = match response.error_for_status() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Request failed for {:?}: {}", &url, e);
            return Err(io::Error::from_raw_os_error(libc::ENETDOWN));
        }
    };

    let writer = File::create(save_path)?;
    match write_response(response, writer) {
        Ok(()) => (),
        Err(err) => {
            log::error!("Failed to write response to file: {}", err);
            // try to clean up the mess
            let _ = fs::remove_file(save_path);
            return Err(err);
        }
    }

    Ok(())
}

fn write_response<W: io::Write>(response: Response, mut writer: W) -> io::Result<()> {
    let data = response.bytes().unwrap_or(Bytes::from_static(b""));
    writer.write_all(&data)?;
    writer.flush()
}
