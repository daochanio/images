use std::sync::Arc;

use crate::settings::Settings;
use crate::usecases::clean_videos::CleanVideos;
use crate::usecases::get_image::GetImage;
use crate::usecases::upload_avatar::UploadAvatar;
use crate::usecases::upload_image::UploadImage;
use crate::usecases::{self};
use crate::{gateways, settings};

pub struct Container {
    pub settings: Arc<Settings>,
    pub upload_image: Arc<UploadImage>,
    pub upload_avatar: Arc<UploadAvatar>,
    pub get_image: Arc<GetImage>,
    pub clean_videos: Arc<CleanVideos>,
}

pub async fn new() -> Container {
    let settings = Arc::new(settings::new());
    let storage = Arc::new(gateways::s3::new(settings.clone()).await);
    let images = Arc::new(gateways::images::new());
    let web = Arc::new(gateways::http::new(settings.clone()));
    let video = Arc::new(gateways::video::new());
    let upload_image = Arc::new(usecases::upload_image::new(
        storage.clone(),
        images.clone(),
        video.clone(),
    ));
    let get_image = Arc::new(usecases::get_image::new(storage.clone()));
    let upload_avatar = Arc::new(usecases::upload_avatar::new(
        web.clone(),
        upload_image.clone(),
        get_image.clone(),
    ));
    let clean_videos = Arc::new(usecases::clean_videos::new(video.clone()));

    Container {
        settings,
        upload_image,
        upload_avatar,
        get_image,
        clean_videos,
    }
}
