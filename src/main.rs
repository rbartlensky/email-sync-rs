use email_sync::{Mailbox, Session};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

macro_rules! HELP_STR {
    () => {
        r#"Email sync utility

Make a local copy of your email in a maildir directory.

USAGE:
    {argv0} [--domain <DOMAIN>]

OPTIONS:
    --domain                The domain name of the IMAP server
    -v, --version           Print version info and exit
    -h, --help              Print help information
"#
    };
}

fn main() -> anyhow::Result<()> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        println!(HELP_STR!(), argv0 = std::env::args().nth(0).unwrap());
        return Ok(());
    }
    if args.contains(["-v", "--version"]) {
        println!("{} {}", std::env::args().nth(0).unwrap(), VERSION);
        return Ok(());
    }
    let domain: Option<String> = args.opt_value_from_str("--domain")?;

    sync_email(domain.as_deref())?;
    Ok(())
}

fn sync_email(domain: Option<&str>) -> anyhow::Result<()> {
    let mut session = Session::new(domain.as_deref())?;
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
