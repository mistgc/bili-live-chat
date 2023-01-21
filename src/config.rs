use serde::{Deserialize, Serialize};
use std::{fs, io::Error};

use crate::Credential;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub credential: Credential,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let se_data = fs::read(path)?;

        Ok(serde_yaml::from_slice(&se_data).unwrap())
    }

    pub fn save_as_file(&self, path: &str) -> Result<(), Error> {
        let de_data = serde_yaml::to_string(self).unwrap();
        fs::write(path, de_data)?;

        Ok(())
    }
}

#[test]
fn test_config_serde() {
    let expected = Config {
        credential: Credential {
            session_data: "111".to_owned(),
            bili_jct: "222".to_owned(),
            buvid3: "333".to_owned(),
        },
    };

    let output = Config::from_file("fixtures/test_config_serde.yml").unwrap();

    assert_eq!(
        output.credential.session_data,
        expected.credential.session_data
    );
    assert_eq!(output.credential.bili_jct, expected.credential.bili_jct);
    assert_eq!(output.credential.buvid3, expected.credential.buvid3);
}
