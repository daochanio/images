use std::{
    fs,
    io::Read,
    io::Write,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use tokio::process::Command;

use crate::{
    common::{format::Format, variant::Variant},
    usecases::gateways::Video,
};

struct VideoImpl {}

pub fn new() -> impl Video {
    VideoImpl {}
}

const DIRECTORY: &str = "/tmp/daochan";

#[async_trait]
impl Video for VideoImpl {
    // TODO:
    // - should variant influence certain params?
    async fn format(
        &self,
        data: &[u8],
        _variant: Variant,
        input_format: Format,
    ) -> Result<(Vec<u8>, Format), String> {
        let input_path = self.get_path(uuid::Uuid::new_v4().to_string(), input_format);

        let output_path = self.get_path(uuid::Uuid::new_v4().to_string(), Format::Mp4);

        self.write(&input_path, &data.to_vec())?;

        let mut child = match Command::new("ffmpeg")
            .arg("-i")
            .arg(&input_path)
            .arg("-y") // overwrite output file if it exists
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-an") // no audio
            .arg("-r") // frame rate
            .arg("16")
            .arg(&output_path)
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return Err(format!("could not spawn video process: {}", e)),
        };

        match child.wait().await {
            Ok(status) => {
                if !status.success() {
                    return Err(format!("video process exited with status: {}", status));
                }
            }
            Err(e) => return Err(format!("video process errored while waiting: {}", e)),
        };

        let buffer = self.read(&output_path)?;

        Ok((buffer, Format::Mp4))
    }

    async fn clean(&self, stale_seconds: u64) -> Result<(), String> {
        let now = SystemTime::now();

        let entries = match fs::read_dir(PathBuf::from(DIRECTORY)) {
            Ok(entries) => entries,
            Err(e) => return Err(format!("could not read directory: {}", e)),
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => return Err(format!("could not read directory entry: {}", e)),
            };

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(e) => return Err(format!("could not read directory entry metadata: {}", e)),
            };

            if metadata.is_file() {
                if let Ok(time) = metadata.modified() {
                    let elapsed_dur = match now.duration_since(time) {
                        Ok(elapsed_dur) => elapsed_dur,
                        Err(e) => return Err(format!("could not get duration: {}", e)),
                    };
                    if elapsed_dur > Duration::from_secs(stale_seconds) {
                        match fs::remove_file(entry.path()) {
                            Ok(_) => (),
                            Err(e) => return Err(format!("could not remove file: {}", e)),
                        }
                    }
                }
            }
        }

        return Ok(());
    }
}

impl VideoImpl {
    fn write(&self, path: &PathBuf, body: &Vec<u8>) -> Result<(), String> {
        let mut file = match fs::File::create(path) {
            Ok(file) => file,
            Err(e) => return Err(format!("could not create file: {}", e)),
        };

        match file.write_all(body) {
            Ok(_) => (),
            Err(e) => return Err(format!("could not write file: {}", e)),
        };

        return Ok(());
    }

    fn read(&self, path: &PathBuf) -> Result<Vec<u8>, String> {
        let mut file = match fs::File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(format!("could not open file: {}", e)),
        };

        let mut buffer = Vec::new();

        match file.read_to_end(&mut buffer) {
            Ok(_) => (),
            Err(e) => return Err(format!("could not read file: {}", e)),
        };

        return Ok(buffer);
    }

    fn get_path(&self, file_name: String, format: Format) -> PathBuf {
        let mut path = PathBuf::from(DIRECTORY);
        path.push(file_name);
        path.set_extension(format.extension());
        return path;
    }
}
