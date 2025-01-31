use imap::{self, Session};
use mailparse::*;
use native_tls::{TlsConnector, TlsStream};
use std::io::{self, Write};
use std::net::TcpStream;
use rpassword::prompt_password;

fn connect(email: String, password: String) -> imap::error::Result<Session<TlsStream<TcpStream>>> {
    let domain = "imap.gmail.com";
    let tls = TlsConnector::builder().build()?;

    // Connect to the IMAP server
    let client = imap::connect((domain, 993), domain, &tls)?;

    // Login to the account
    client.login(&email, &password)
        .map_err(|(err, _)| err)
}

fn fetch_recent_emails(
    mut session: Session<TlsStream<TcpStream>>,
    count: usize,
) -> imap::error::Result<()> {
    session.select("INBOX")?;
    let messages = session.search("ALL")?;
    let total_messages = messages.len();
    
    let start = if total_messages > count {
        total_messages - count + 1
    } else {
        1
    };
    let end = total_messages;

    let fetch_results = session.fetch(format!("{}:{}", start, end), "RFC822")?;

    for message in fetch_results.iter().rev() {
        if let Some(body) = message.body() {
            if let Ok(parsed) = parse_mail(&body) {
                print_email(&parsed);
                println!("{}", "-".repeat(50));
            }
        }
    }

    Ok(())
}

fn print_email(parsed_mail: &ParsedMail) {
    println!("From: {}", parsed_mail.headers.get_first_value("From").unwrap_or_default());
    println!("Subject: {}", parsed_mail.headers.get_first_value("Subject").unwrap_or_default());
    println!("Date: {}", parsed_mail.headers.get_first_value("Date").unwrap_or_default());

    if let Ok(body) = parsed_mail.get_body() {
        println!("\n{}", body.trim());
    }
}

fn main() -> imap::error::Result<()> {
    // Get email and password interactively
    print!("Email: ");
    io::stdout().flush().unwrap();
    let mut email = String::new();
    io::stdin().read_line(&mut email).expect("Failed to read email");
    let email = email.trim().to_string();

    let password = prompt_password("Password: ").expect("Failed to read password");

    // Connect and authenticate
    let session = match connect(email, password) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Connection error: {}", e);
            return Ok(());
        }
    };

    // Fetch and display 5 most recent emails
    if let Err(e) = fetch_recent_emails(session, 5) {
        eprintln!("Error fetching emails: {}", e);
    }

    Ok(())
}
