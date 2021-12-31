use clap::ArgEnum;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum InitType {
    Shell,
    Postgres,
    Mysql,
}

#[derive(Serialize, Deserialize)]
pub struct RepoConfig {
    pub mode: InitType,
}
