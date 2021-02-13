use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{SendableEmail, SmtpClient, SmtpTransport, Transport};
use lettre_email::EmailBuilder;

pub fn send_email(
    email: &String,
    name: &String,
    subject: &String,
    body: &String,
    email_address: &String,
    email_password: &String,
    email_provider:&String
) {
    let email = EmailBuilder::new()
        // Addresses can be specified by the tuple (email, alias)
        .to((email, name))
        .from(email_address.clone())
        .subject(subject)
        .html(body)
        .build()
        .unwrap();
    let email: SendableEmail = email.into();

    let credentials = Credentials::new(email_address.into(), email_password.into());
    let client = SmtpClient::new_simple(email_provider)
        .unwrap()
        .credentials(credentials)
        .authentication_mechanism(Mechanism::Plain);

    // build the Transport
    let mut mailer = SmtpTransport::new(client);

    // Send the email
    match mailer.send(email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => eprintln!("Could not send email: {:?}", e),
    }

    mailer.close();
}
