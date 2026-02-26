/// Serviço de upload com MinIO (compatível S3)
///
/// Funções:
/// - Gerar presigned URLs para upload direto do browser
/// - Verificar existência de objetos
/// - Deletar objetos
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::Region;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadConfig {
    pub endpoint: String,
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub public_url: String,
}

impl Default for UploadConfig {
    fn default() -> Self {
        crate::env::load();
        Self {
            endpoint: std::env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            bucket_name: std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "loco-fast-store".to_string()),
            access_key: std::env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            region: std::env::var("MINIO_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            public_url: std::env::var("MINIO_PUBLIC_URL")
                .unwrap_or_else(|_| "http://localhost:9000/loco-fast-store".to_string()),
        }
    }
}

pub struct UploadService {
    bucket: Box<Bucket>,
    pub public_url: String,
}

impl UploadService {
    /// Inicializa o serviço de upload
    pub fn new(config: &UploadConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let region = Region::Custom {
            region: config.region.clone(),
            endpoint: config.endpoint.clone(),
        };

        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )?;

        let bucket = Bucket::new(&config.bucket_name, region, credentials)?
            .with_path_style();

        Ok(Self {
            bucket,
            public_url: config.public_url.clone(),
        })
    }

    /// Gera presigned URL para upload (PUT)
    pub async fn presigned_upload_url(
        &self,
        key: &str,
        expiry_secs: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.bucket.presign_put(key, expiry_secs, None, None).await?;
        Ok(url)
    }

    /// Gera presigned URL para download (GET)
    pub async fn presigned_download_url(
        &self,
        key: &str,
        expiry_secs: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.bucket.presign_get(key, expiry_secs, None).await?;
        Ok(url)
    }

    /// Faz upload de bytes diretamente
    pub async fn upload_bytes(
        &self,
        key: &str,
        content: &[u8],
        content_type: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.bucket.put_object_with_content_type(key, content, content_type).await?;
        Ok(format!("{}/{}", self.public_url, key))
    }

    /// Deleta objeto
    pub async fn delete_object(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.bucket.delete_object(key).await?;
        Ok(())
    }

    /// Gera key para imagem de produto
    pub fn product_image_key(product_pid: &str, filename: &str) -> String {
        format!("products/{}/{}", product_pid, filename)
    }

    /// Gera key para imagem de categoria
    pub fn category_image_key(category_pid: &str, filename: &str) -> String {
        format!("categories/{}/{}", category_pid, filename)
    }

    /// Gera key para assets (logo, favicon, etc)
    pub fn store_asset_key(filename: &str) -> String {
        format!("assets/{}", filename)
    }
}
