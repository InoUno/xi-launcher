use std::{collections::HashMap, fs::File, path::PathBuf};

use tauri::{async_runtime::RwLock, App, Manager, State};
use tokio_util::sync::CancellationToken;

use crate::{
    check_game::VersionsInfo,
    config::{launcher::LauncherConfig, profiles::Profiles},
    util::load_json_or_default,
};

pub type AppState<'a> = State<'a, RwLock<AppStateData>>;

pub struct AppStateData {
    pub local_data_dir: PathBuf,
    pub config: LauncherConfig,
    pub profiles: Profiles,
    pub update_cache: HashMap<String, VersionsInfo>,
    pub ongoing_tasks: HashMap<u32, CancellationToken>,
}

impl AppStateData {
    pub async fn new(app: &App) -> anyhow::Result<Self> {
        let local_data_dir = app.path().app_local_data_dir().unwrap();

        let config = load_json_or_default(&LauncherConfig::get_path(&local_data_dir));

        let profiles_config = load_json_or_default(&Profiles::get_path(&local_data_dir));

        Ok(Self {
            local_data_dir,
            config,
            profiles: profiles_config,
            update_cache: Default::default(),
            ongoing_tasks: Default::default(),
        })
    }

    pub fn save_configs(&self) -> anyhow::Result<()> {
        let file = File::create(LauncherConfig::get_path(&self.local_data_dir))?;
        serde_json::to_writer(file, &self.config)?;

        let file = File::create(Profiles::get_path(&self.local_data_dir))?;
        serde_json::to_writer(file, &self.profiles)?;

        Ok(())
    }
}
