use anyhow::Context;

pub type ImapSession = imap::Session<native_tls::TlsStream<std::net::TcpStream>>;

/// Represents an IMAP session.
pub struct Session {
    session: ImapSession,
}

impl Session {
    /// Connects to an IMAP server, and logs in.
    pub fn new(
        domain: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, anyhow::Error> {
        let session = Session::connect(domain, port, username, password)?;
        Ok(Self { session })
    }

    fn connect(
        domain: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<ImapSession, anyhow::Error> {
        let (tls, name) = if domain != "127.0.0.1" {
            let tls = native_tls::TlsConnector::builder()
                .build()
                .context("Failed to create TLS connector.")?;
            (tls, domain)
        } else {
            let tls = native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
                .unwrap();
            (tls, "imap.example.com")
        };
        let client = imap::connect((domain, port), name, &tls)
            .with_context(|| format!("IMAP: Failed to connect to {}:{}.", domain, port))?;
        let session = client
            .login(username, password)
            .map_err(|e| e.0)
            .context("Failed to login.")?;
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
