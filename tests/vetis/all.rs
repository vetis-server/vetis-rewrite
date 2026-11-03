use crate::common::{CA_CERT, VetisConfig};
use caramelo::{expect, matchers::eq};
use deboa::{
    cert::{CertificateExt as _, ContentEncoding},
    request,
};
use deboa_tokio::cert::DeboaCertificate;
use http::Version;
use std::path::Path;
use tokio::fs::read_to_string;
use vetis::{VetisServer as _, VetisTestResult};
#[allow(unused_imports)]
use vetis_flash::FlashPathConfig;
#[allow(unused_imports)]
use vetis_rewrite::regex::HookConfig;
use vetis_tokio::Vetis;

#[tokio::test]
async fn test_from_file() -> VetisTestResult<()> {
    let config_path = Path::new("files/vetis-rewrite.yaml");
    if config_path.exists() {
        let file = read_to_string(&config_path).await;
        if let Ok(file) = file {
            let config = serde_yaml_ng::from_str::<VetisConfig>(&file);
            if let Ok(config) = config {
                let config_clone = config
                    .server()
                    .clone();

                let mut server = Vetis::from_config(config_clone).await?;
                server
                    .start()
                    .await?;

                let cert = tokio::fs::read(CA_CERT).await?;
                let client = deboa_tokio::Client::builder()
                    .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
                    .build();

                // Test regex rewrite
                let request = request::get("https://localhost:8443/hello/world")?
                    .version(Version::HTTP_2)
                    .send_with(&client)
                    .await?;

                expect(request.status()).to_be(eq(http::StatusCode::OK));
                expect(
                    request
                        .text()
                        .await?,
                )
                .to_be(eq("Hello VeTiS!"));

                // Test path replace rewrite
                let request = request::get("https://localhost:8443/app/service")?
                    .version(Version::HTTP_2)
                    .send_with(&client)
                    .await?;

                expect(request.status()).to_be(eq(http::StatusCode::OK));
                expect(
                    request
                        .text()
                        .await?,
                )
                .to_be(eq("Well done!"));

                // Test strip prefix rewrite
                let request = request::get("https://localhost:8443/home/service")?
                    .version(Version::HTTP_2)
                    .send_with(&client)
                    .await?;

                expect(request.status()).to_be(eq(http::StatusCode::OK));
                expect(
                    request
                        .text()
                        .await?,
                )
                .to_be(eq("Welcome to Services Portal!"));

                // Test strip suffix rewrite
                let request = request::get("https://localhost:8443/portal/home")?
                    .version(Version::HTTP_2)
                    .send_with(&client)
                    .await?;

                expect(request.status()).to_be(eq(http::StatusCode::OK));
                expect(
                    request
                        .text()
                        .await?,
                )
                .to_be(eq("Welcome to Portal!"));

                server
                    .stop()
                    .await?;
            } else {
                eprintln!(
                    "Could not read : {}",
                    config
                        .err()
                        .unwrap()
                );
            }
        } else {
            eprintln!("Failed to start server: {}", config_path.display());
        }
    } else {
        eprintln!("Failed to start server: Config file does not exist: {}", config_path.display());
    }

    Ok(())
}
