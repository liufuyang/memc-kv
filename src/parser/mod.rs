pub mod ascii;

/// A set command from client.
#[derive(Clone, Debug, PartialEq)]
pub enum Cmd {
    CmdSet {
        /// The key.
        key: Vec<u8>,
        /// Flag for this key.
        ///
        /// Defaults to 0.
        flag: u32,
        /// ttl for this key.
        ///
        /// Defaults to 0.
        ttl: u32,
        /// Length of data
        len: u32,
        /// noreply
        noreply: Option<bool>,
    },

    /// A get command from client.
    CmdGet {
        /// The key.
        key: Vec<u8>,
    },
}
