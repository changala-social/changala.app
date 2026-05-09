//! SMTP email sender for OTP delivery.
//!
//! Uses `lettre` for async SMTP with STARTTLS (Gmail app password compatible).
//! When SMTP is not configured (SmtpConfig is None), OTPs are logged to stdout.

use serde::Deserialize;

/// SMTP configuration, deserialized from `[changala.smtp]` in atrg.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    #[serde(default = "default_encryption")]
    pub encryption: String,
}

fn default_encryption() -> String {
    "starttls".to_string()
}

/// Send an OTP email. Returns Ok(()) on success.
/// If smtp_config is None, logs the OTP instead of sending.
pub async fn send_otp_email(
    smtp: Option<&SmtpConfig>,
    to_email: &str,
    otp: &str,
) -> anyhow::Result<()> {
    let Some(config) = smtp else {
        tracing::info!(
            email = %to_email,
            otp = %otp,
            "OTP generated (SMTP not configured — dev mode, logged to stdout)"
        );
        return Ok(());
    };

    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

    let email = Message::builder()
        .from(config.from.parse()?)
        .to(to_email.parse()?)
        .subject("Changala — Your verification code")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Your verification code is: {}\n\nThis code expires in 10 minutes.\nIf you did not request this, ignore this email.",
            otp
        ))?;

    let transport = match config.encryption.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
            .port(config.port)
            .credentials(Credentials::new(
                config.username.clone(),
                config.password.clone(),
            ))
            .build(),
        "none" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
            .port(config.port)
            .credentials(Credentials::new(
                config.username.clone(),
                config.password.clone(),
            ))
            .build(),
        _ => {
            // Default: starttls
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
                .port(config.port)
                .credentials(Credentials::new(
                    config.username.clone(),
                    config.password.clone(),
                ))
                .build()
        }
    };

    transport.send(email).await?;
    tracing::info!(email = %to_email, "OTP email sent successfully");
    Ok(())
}
