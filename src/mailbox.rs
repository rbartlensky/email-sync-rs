use crate::{
    utils::{concat, convert_to_u32, convert_to_u8},
    Session,
};

use std::{
    fs::{read, write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use imap::types::{Fetch, ZeroCopy};

/// Represents a mailbox, which can be saved to disk into a maildir.
pub struct Mailbox<'s> {
    session: &'s mut Session,
    maildir: maildir::Maildir,
    uid_validity: u32,
    last_uid: Option<u32>,
}

impl<'s> Mailbox<'s> {
    /// Creates a new mailbox based on a session, and a name returned by
    /// `list_all`.
    pub fn new(session: &'s mut Session, mailbox: &str) -> anyhow::Result<Self> {
        let maildir = prepare_maildir(mailbox)?;
        let saved_uid = saved_uid(maildir.path())?;
        let uid_validity = session
            .select(mailbox)?
            .uid_validity
            .context("Server doesn't support UIDVALIDITY.")?;
        if let Some(suid) = saved_uid.uid_validity {
            if suid != uid_validity {
                unimplemented!(
                    "UIDVALIDITY for mailbox changed. Expected: {}, found: {}.",
                    suid,
                    uid_validity
                );
            }
        }
        Ok(Mailbox {
            session,
            maildir,
            uid_validity,
            last_uid: saved_uid.last_uid,
        })
    }

    /// Returns a list of all messages that cannot be found in the local
    /// maildir.
    pub fn messages(&mut self) -> anyhow::Result<ZeroCopy<Vec<Fetch>>> {
        let uids = self.search_uids()?;
        // fetch their body and envelope
        let res = self.session.inner().uid_fetch(uids, "(RFC822 ENVELOPE)")?;
        Ok(res)
    }

    /// Saves a particular message to the local maildir.
    pub fn save(&mut self, message: &Fetch) -> anyhow::Result<()> {
        let uid = message.uid.unwrap();
        self.maildir
            // we don't store any flags for now
            .store_cur_with_flags(message.body().unwrap(), "")
            .unwrap();

        // update our last_uid value
        let uid_validity = convert_to_u8(self.uid_validity);
        let last_uid = convert_to_u8(uid);
        write(
            self.maildir.path().join("last_uid"),
            concat(uid_validity, last_uid),
        )
        .context("Failed to write.")?;
        Ok(())
    }

    /// Returns a query string for the fetch command. If we have a last_uid,
    /// then we return `last_uid+1:*`. Otherwise we fetch all messages.
    fn search_uids(&mut self) -> anyhow::Result<String> {
        let res = if let Some(last_uid) = self.last_uid {
            format!("{}:{}", last_uid.wrapping_add(1), std::u32::MAX)
        } else {
            self.session
                .inner()
                .uid_search("ALL")?
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",")
        };
        Ok(res)
    }

    /// Returns the path to the root of the local maildir.
    pub fn path(&self) -> &Path {
        self.maildir.path()
    }
}

/// Sets up the local maildir for a particular IMAP folder.
fn prepare_maildir(name: &str) -> anyhow::Result<maildir::Maildir> {
    let path = format!("/home/robert/.ecrs/{}/", name);
    let maildir_path = PathBuf::from(&path);
    let maildir = maildir::Maildir::from(maildir_path);
    maildir
        .create_dirs()
        .with_context(|| format!("Failed to create maildir dir: {:?}", path))?;
    Ok(maildir)
}

#[derive(Default)]
struct SavedUid {
    uid_validity: Option<u32>,
    last_uid: Option<u32>,
}

/// Checks if we saved any information about `uid_validity` and `last_uid`
/// into the local maildir.
fn saved_uid(maildir_path: &Path) -> anyhow::Result<SavedUid> {
    use std::convert::TryInto;

    let path = maildir_path.join("last_uid");
    let saved_uid = match read(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Default::default()),
        e => e.with_context(|| format!("Failed to read from {}", path.display(),))?,
    };

    if saved_uid.is_empty() {
        // this is our first time backing things up
        Ok(Default::default())
    } else {
        if saved_uid.len() != 8 {
            return Err(anyhow!(
                "Contents of {} are not valid. Expected size to be 8 bytes, got: {} ",
                path.display(),
                saved_uid.len(),
            ));
        }
        Ok(SavedUid {
            uid_validity: Some(convert_to_u32(saved_uid[0..4].try_into().unwrap())),
            last_uid: Some(convert_to_u32(saved_uid[4..8].try_into().unwrap())),
        })
    }
}
