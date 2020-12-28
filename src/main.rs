use email_sync::{Mailbox, Session};

fn main() -> anyhow::Result<()> {
    let mut session = Session::new()?;
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
