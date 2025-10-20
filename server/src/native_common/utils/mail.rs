use lettre::address::Address;
use lettre::message::header::ContentType;
use lettre::message::IntoBody;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::str::FromStr;

pub async fn send_mail<S: Into<String>, T: IntoBody>(
    mail_host: &str,
    mail_port: u16,
    starttls: bool,
    username: String,
    password: String,
    from_name: Option<String>,
    from_addr: &str,
    to_name: Option<String>,
    to_addr: &str,
    subject: S,
    body: T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let email = Message::builder()
        .from(Mailbox {
            name: from_name,
            email: Address::from_str(from_addr)?,
        })
        .to(Mailbox {
            name: to_name,
            email: Address::from_str(to_addr)?,
        })
        .header(ContentType::TEXT_HTML)
        .subject(subject)
        .body(body)?;
    let credentials = Credentials::new(username, password);
    let builder = if starttls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(mail_host)?
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(mail_host)?
    };
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        builder.port(mail_port).credentials(credentials).build();
    mailer.send(email).await?;
    return Ok(());
}
