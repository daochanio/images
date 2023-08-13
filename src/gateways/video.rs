use std::{
    fs,
    io::Read,
    io::Write,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Result};
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
    ) -> Result<(Vec<u8>, Format)> {
        let input_path = self.get_path(uuid::Uuid::new_v4().to_string(), input_format);

        let output_path = self.get_path(uuid::Uuid::new_v4().to_string(), Format::Mp4);

        self.write(&input_path, &data.to_vec())?;

        let mut child = Command::new("ffmpeg")
            .arg("-i")
            .arg(&input_path)
            .arg("-y") // overwrite output file if it exists
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-an") // no audio
            .arg("-r") // frame rate
            .arg("16")
            .arg("-crf") // quality
            .arg("23")
            .arg("-preset") // speed
            .arg("slow")
            .arg("-c:v") // codec
            .arg("libx264")
            .arg("-movflags") // fast start
            .arg("+faststart")
            .arg("-pix_fmt") // pixel format
            .arg("yuv420p") // required for safari and firefox
            .arg(&output_path)
            .spawn()
            .map_err(|e| anyhow!("could not spawn video process: {}", e))?;

        let status = child
            .wait()
            .await
            .map_err(|e| anyhow!("video process errored: {}", e))?;

        if !status.success() {
            return Err(anyhow!("video process exited with status: {}", status));
        }

        let buffer = self.read(&output_path)?;

        Ok((buffer, Format::Mp4))
    }

    async fn clean(&self, stale_seconds: u64) -> Result<()> {
        let now = SystemTime::now();

        let entries = fs::read_dir(PathBuf::from(DIRECTORY))
            .map_err(|e| anyhow!("could not read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("could not read directory entry: {}", e))?;

            let metadata = entry
                .metadata()
                .map_err(|e| anyhow!("could not read directory entry metadata: {}", e))?;

            if metadata.is_file() {
                if let Ok(time) = metadata.modified() {
                    let elapsed_dur = now
                        .duration_since(time)
                        .map_err(|e| anyhow!("could not get duration: {}", e))?;
                    if elapsed_dur > Duration::from_secs(stale_seconds) {
                        fs::remove_file(entry.path())
                            .map_err(|e| anyhow!("could not remove file: {}", e))?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl VideoImpl {
    fn write(&self, path: &PathBuf, body: &Vec<u8>) -> Result<()> {
        let mut file =
            fs::File::create(path).map_err(|e| anyhow!("could not create file: {}", e))?;

        file.write_all(body)
            .map_err(|e| anyhow!("could not write file: {}", e))?;

        Ok(())
    }

    fn read(&self, path: &PathBuf) -> Result<Vec<u8>> {
        let mut file = fs::File::open(path).map_err(|e| anyhow!("could not open file: {}", e))?;

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .map_err(|e| anyhow!("could not read file: {}", e))?;

        Ok(buffer)
    }

    fn get_path(&self, file_name: String, format: Format) -> PathBuf {
        let mut path = PathBuf::from(DIRECTORY);

        path.push(file_name);
        path.set_extension(format.extension());

        path
    }
}
