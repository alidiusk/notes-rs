use chrono::{DateTime, Duration, Utc};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use directories::ProjectDirs;
use failure::ResultExt;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use text_io::read;

/// Application configuration struct.
///
/// `key` stores the option of a hashed password, hashed using sha3_256.
/// If `key` is `None`, then the database is not password encrypted.
/// Otherwise, it contains a `Some(String)`, which is used to encrypt the
/// text fields of the database.
///
/// `session_expiration` stores the expiration time of the current session.
/// Sessions currently are hardcoded to last 30 minutes until the user will
/// have to enter their password to renew their session.
pub struct Config {
    key: Option<String>,
    session_expiration: DateTime<Utc>,
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Config", 2)?;
        state.serialize_field("key", &self.pass_hash)?;
    }
}

impl Config {
    /// Create a new configuration struct.
    pub fn new(password: Option<String>) -> Self {
        let key = if password.is_some() {
            let mut hasher = Sha3::sha3_256();
            hasher.input_str(&password.unwrap());

            // in hex
            Some(hasher.result_str())
        } else {
            None
        };

        // Not leaving it to caller
        let session_expiration = Utc::now() + Duration::minutes(30);

        Config {
            key,
            session_expiration,
        }
    }

    /// Commence an initial configuration dialogue with the user; they can eitheer
    /// choose to encrypt their notes using a password hash, or opt out.
    pub fn init_config_dialogue() -> Result<(), failure::Error> {
        loop {
            print!("Welcome to notes! Would you like your notes to be password encrypted [y/n]? ");
            let ans: String = read!("{}\n");
            let ans = ans.as_str().to_lowercase();
            match ans.as_str() {
                "y" => {
                    Config::new_config(true)?;
                    break;
                }
                "n" => {
                    Config::new_config(false)?;
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    /// Creates a new configuration file.
    fn new_config(encrypted: bool) -> Result<(), failure::Error> {
        if encrypted {
            Config::request_new_password()?;
        } else {
            let config = Config::new(None);
            Config::write_to_config(config)?;
        }

        Ok(())
    }

    /// Configuration dialogue to request a new password from the user. Used for initial
    /// configuration setup.
    fn request_new_password() -> Result<(), failure::Error> {
        print!("Enter a password to encrypt your notes: ");
        let password: String = read!("{}\n");
        let config = Config::new(Some(password));
        Config::write_to_config(config)?;
        Ok(())
    }

    /// Checks whether the configuration file exists.
    pub fn config_exists() -> Result<bool, failure::Error> {
        let project_dir = match ProjectDirs::from("", "", "Notes") {
            None => Err(failure::err_msg("Could not open local data directory."))
                .context("Could not get access to database.")?,
            Some(d) => d,
        };

        let dir = project_dir.config_dir();

        Ok(dir.exists())
    }

    /// Returns the path of the configuration file. Creates the directory if it
    /// does not exist.
    fn config_path() -> Result<std::path::PathBuf, failure::Error> {
        let project_dir = match ProjectDirs::from("", "", "Notes") {
            None => Err(failure::err_msg("Could not open local data directory."))
                .context("Could not get access to database.")?,
            Some(d) => d,
        };

        let dir = project_dir.config_dir();
        if !dir.exists() {
            std::fs::create_dir(dir).with_context(|_| {
                format!("could not create config: `{}`", dir.to_str().unwrap())
            })?;
        }

        let path = dir.join("config.yaml");

        Ok(path)
    }

    /// Writes a given config struct to the configuration file.
    fn write_to_config(config: Config) -> Result<(), failure::Error> {
        let path = Config::config_path()?;
        std::fs::write(&path, serde_yaml::to_string(&config)?)?;
        Ok(())
    }

    /// Returns a config struct derived from the configuration file.
    fn load_config() -> Result<Self, failure::Error> {
        let path = Config::config_path()?;

        let contents = std::fs::read_to_string(path).context("Could not read config file.")?;
        Ok(serde_yaml::from_str::<Config>(&contents).context("Could not serialize config.")?)
    }

    /// Returns a bool indicating if the database is encrypted.
    pub fn is_encrypted() -> Result<bool, failure::Error> {
        let config = Config::load_config()?;

        Ok(config.key.is_some())
    }

    /// Checks whether a given password, when hashed, matches that of the configuration file.
    pub fn correct_hash(password: String) -> Result<bool, failure::Error> {
        let mut hasher = Sha3::sha3_256();
        hasher.input_str(&password);

        let key = hasher.result_str();

        let config = Config::load_config()?;

        Ok(config.key == Some(pass_hash))
    }

    /// Checks whether the current session is expired.
    pub fn session_expired() -> Result<bool, failure::Error> {
        let config = Config::load_config()?;

        // No sessions if there isn't a password.
        if config.key.is_none() {
            Ok(false)
        } else {
            Ok(config.session_expiration <= Utc::now())
        }
    }

    /// Restores the session for a duration of 30 minutes.
    pub fn restore_session() -> Result<(), failure::Error> {
        let mut config = Config::load_config()?;
        config.session_expiration = Utc::now() + Duration::minutes(30);
        Config::write_to_config(config)?;

        Ok(())
    }
}
