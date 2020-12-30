use anyhow::Context;
use email_sync::{Mailbox, Session};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP_STR: &str = r#"Email sync utility

Make a local copy of your email in a maildir directory.

USAGE:
    email-sync <OPTIONS>

OPTIONS:
    -d, --domain                  The domain name of the IMAP server
    -p, --port                    The port of the IMAP server (Default: 993)
    -u, --username                The username to authenticate as
    -v, --version                 Print version info and exit
    -h, --help                    Print help information
"#;

fn main() -> anyhow::Result<()> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        println!("{}", HELP_STR);
        return Ok(());
    }
    if args.contains(["-v", "--version"]) {
        println!("email-sync {}", VERSION);
        return Ok(());
    }
    let domain: String = args
        .opt_value_from_str(["-d", "--domain"])?
        .context("Missing argument `--domain`.")?;
    let username: String = args
        .opt_value_from_str(["-u", "--username"])?
        .context("Missing argument `--username`.")?;
    let port: u16 = args
        .opt_value_from_fn(["-p", "--port"], str::parse::<u16>)?
        .unwrap_or(993);
    let password = rpassword::prompt_password_stdout("Password: ")
        .context("Failed to read password from user.")?;

    sync_email(&domain, port, &username, &password)?;
    Ok(())
}

fn sync_email(domain: &str, port: u16, username: &str, password: &str) -> anyhow::Result<()> {
    let mut session = Session::new(domain, port, username, password)?;
    let list = session.list_all()?;
    for name in list {
        let mut mb = Mailbox::new(&mut session, &name)?;

        let messages = mb.messages()?;
        if messages.is_empty() {
            println!("Mailbox: {} is up-to-date.", name);
            continue;
        }

        println!("Backing up mailbox: {} to {}", name, mb.path().display());
        print!("0/{}", messages.len());
        for (i, m) in messages.iter().enumerate() {
            mb.save(&m)?;
            print!("\r{}/{}", i + 1, messages.len());
        }
        println!();
    }
    Ok(())
}
