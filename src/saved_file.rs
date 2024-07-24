use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use tokio::{fs, io::AsyncWriteExt};

#[async_trait]
pub trait SavedFile: Serialize + DeserializeOwned {
    const PATH: &'static str;

    async fn load() -> Result<Self, Box<dyn Error>> {
        let path = dirs::home_dir().unwrap().join(Self::PATH);
        let content = fs::read_to_string(path).await?;

        Ok(serde_json::from_str(&content)?)
    }

    async fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = dirs::home_dir().unwrap().join(Self::PATH);
        let content = serde_json::to_string(&self)?;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .await?;

        Ok(file.write_all(content.as_bytes()).await?)
    }
}
