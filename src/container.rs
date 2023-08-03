use std::sync::Arc;

use crate::settings::Settings;
use crate::usecases;
use crate::usecases::upload_avatar::UploadAvatar;
use crate::usecases::upload_image::UploadImage;
use crate::{gateways, settings};

pub struct Container {
    pub settings: Arc<Settings>,
    pub upload_image: Arc<UploadImage>,
    pub upload_avatar: Arc<UploadAvatar>,
}

pub async fn new() -> Container {
    let settings = Arc::new(settings::new());
    let s3 = Arc::new(gateways::s3::new(settings.clone()).await);
    let images = Arc::new(gateways::images::new());
    let upload_image = Arc::new(usecases::upload_image::new(s3.clone(), images.clone()));
    let upload_avatar = Arc::new(usecases::upload_avatar::new(s3.clone(), images.clone()));

    Container {
        settings,
        upload_image,
        upload_avatar,
    }
}
