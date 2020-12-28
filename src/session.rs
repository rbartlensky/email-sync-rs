use anyhow::Context;

pub type ImapSession = imap::Session<native_tls::TlsStream<std::net::TcpStream>>;

/// Represents an IMAP session.
pub struct Session {
    session: ImapSession,
}

impl Session {
    /// Connects to a testing IMAP server on 127.0.0.1:3993.
    pub fn new() -> Result<Self, anyhow::Error> {
        let session = Session::connect()?;
        Ok(Self { session })
    }

    fn connect() -> Result<ImapSession, anyhow::Error> {
        let domain = "imap.example.com";
        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap();
        // let tls = native_tls::TlsConnector::builder()
        //     .build()
        //     .context("Failed to create TLS connector.")?;
        let client = imap::connect(("127.0.0.1", 3993), domain, &tls)
            .with_context(|| format!("IMAP: Failed to connect to {}:{}.", domain, 993))?;
        let mut session = client
            .login("inbox@localhost", "")
            .map_err(|e| e.0)
            .context("Failed to login.")?;
        session
            .select("INBOX")
            .context("No such folder named INBOX.")?;
        Ok(session)
    }

    /// Returns a list of all folders as specified by the IMAP server.
    pub fn list_all(&mut self) -> anyhow::Result<Vec<String>> {
        let mailboxes = self
            .session
            .list(None, Some("*"))
            .context("Failed to list mailboxes.")?;
        Ok(mailboxes.iter().map(|n| n.name().to_string()).collect())
    }

    /// Selects a particular mailbox, which can be saved to disk.
    pub fn select(&mut self, mailbox: &str) -> anyhow::Result<imap::types::Mailbox> {
        self.session
            .select(mailbox)
            .with_context(|| format!("Failed to SELECT {}", mailbox))
    }

    /// Returns a reference to the inner `imap::Session`.
    pub fn inner(&mut self) -> &mut ImapSession {
        &mut self.session
    }
}
