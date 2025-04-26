use std::{os::windows::process::CommandExt, path::PathBuf, process::Command};

use anyhow::{anyhow, Context};

use tauri::{path::BaseDirectory, AppHandle, Manager};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::config::profiles::{AuthKind, Profile};

pub async fn launch_game(
    profile: &Profile,
    provided_password: Option<String>,
) -> anyhow::Result<()> {
    let mut exe = profile.install.try_get_ashita_dir()?;
    exe.push("Ashita-cli.exe");

    if !exe.exists() {
        return Err(anyhow!("Ashita executable not found at: {}", exe.display()));
    }

    let profile_filename = profile.get_profile_filename();
    update_gamepad_config(profile).await?;

    let working_dir = exe.parent().unwrap().to_path_buf();

    let mut cmd = Command::new("cmd");
    cmd.creation_flags(0x00000008) // DETACHED_PROCESS
        .current_dir(working_dir)
        .arg("/C")
        .arg(exe)
        .arg(format!("{}.ini", profile_filename));

    if !profile.manual_auth && !profile.is_retail {
        let account_name = profile
            .account_name
            .as_ref()
            .cloned()
            .ok_or(anyhow!("Missing username."))?;

        match profile.auth_kind {
            AuthKind::Token => {
                if let Some(password) = provided_password {
                    cmd.arg("--user")
                        .arg(&account_name)
                        .arg("--pass")
                        .arg(&password);
                }

                cmd.arg("--tokenfile").arg(
                    profile
                        .get_token_path()
                        .ok_or_else(|| anyhow!("Missing token path."))?
                        .to_str()
                        .unwrap(),
                );
            }
            AuthKind::Password => {
                let Some(stored_password) = &profile.password else {
                    return Err(anyhow!("No password stored locally."));
                };

                cmd.arg("--user")
                    .arg(&account_name)
                    .arg("--pass")
                    .arg(&stored_password);
            }
            AuthKind::ManualPassword => {
                let Some(password) = provided_password else {
                    return Err(anyhow!("Expected a provided password."));
                };

                cmd.arg("--user")
                    .arg(&account_name)
                    .arg("--pass")
                    .arg(&password);
            }
        }
    }

    tracing::info!("Launching Ashita with profile: {}", profile_filename);
    tracing::info!("Command: {:#?}", cmd);

    let mut _child = cmd.spawn().context("Failed to start Ashita")?;

    tauri::async_runtime::spawn(async move {
        if let Ok(exit_code) = _child.wait() {
            tracing::debug!("Got exit code: {exit_code}");
        }
    });

    Ok(())
}

pub async fn update_ashita_files(profile: &Profile, app_handle: &AppHandle) -> anyhow::Result<()> {
    let profile_filename = profile.get_profile_filename();
    let server_folder_name = profile.get_server_filename();

    make_script_file(profile, &profile_filename).await?;

    let ashita_directory = profile.install.try_get_ashita_dir()?;
    let ini_file_path = ashita_directory.join(format!("config/boot/{}.ini", &profile_filename));

    // Update profile ini file if it exists already
    let mut ashita_ini = if ini_file_path.exists() {
        ini::Ini::load_from_file(&ini_file_path).with_context(|| {
            anyhow!(
                "Could not load Ashita profile ini file: {}",
                ini_file_path.display()
            )
        })?
    } else {
        // Else start from a base ini file
        let ini_resource_path = app_handle
            .path()
            .resolve("resources/ashita_base.ini", BaseDirectory::Resource)
            .with_context(|| format!("Could not resolve Ashita base ini resource file"))?;

        ini::Ini::load_from_file(ini_resource_path)
            .with_context(|| anyhow!("Could not load Ashita base resource file."))?
    };

    profile.name.as_ref().map(|name| {
        ashita_ini
            .with_section(Some("ashita.launcher"))
            .set("name", name);
    });

    // Boot setup
    ashita_ini
        .with_section(Some("ashita.boot"))
        .set("script", format!("{}.txt", profile_filename));

    let game_directory = profile
        .install
        .directory
        .as_ref()
        .ok_or_else(|| anyhow!("Missing game directory."))?;

    if profile.is_retail {
        let pol_path = game_directory.join("PlayOnlineViewer\\pol.exe");

        ashita_ini
            .with_section(Some("ashita.boot"))
            .set("file", pol_path.to_str().unwrap_or_default())
            .set("command", "/game eAZcFcB");
    } else {
        let mut command = vec![];

        profile.server.as_ref().map(|server| {
            command.push("--server".to_string());
            command.push(server.clone());
        });

        if profile.hairpin {
            command.push("--hairpin".to_string());
        }

        let bootloader_path =
            ashita_directory.join(format!("bootloader\\{}\\xiloader.exe", &server_folder_name));

        if !command.is_empty() {
            ashita_ini
                .with_section(Some("ashita.boot"))
                .set("command", command.join(" "))
                .set("file", bootloader_path.to_str().unwrap());
        }
    }

    // Sandbox paths
    ashita_ini
        .with_section(Some("sandbox.paths"))
        .set(
            "pol",
            game_directory
                .join("PlayOnlineViewer")
                .to_str()
                .unwrap_or_default(),
        )
        .set(
            "ffxi",
            game_directory
                .join("FINAL FANTASY XI")
                .to_str()
                .unwrap_or_default(),
        );

    // FFXI settings
    ashita_ini
        .with_section(Some("ffxi.registry"))
        .set("0001", profile.resolution.width.to_string())
        .set("0002", profile.resolution.height.to_string())
        .set("0003", profile.background_resolution.width.to_string())
        .set("0004", profile.background_resolution.height.to_string())
        .set("0037", profile.menu_resolution.width.to_string())
        .set("0038", profile.menu_resolution.height.to_string());

    // Pivot
    generate_pivot_ini(profile, &profile_filename, &server_folder_name).await?;

    ashita_ini
        .with_section(Some("ashita.polplugins"))
        .set("pivot", "1");

    ashita_ini
        .with_section(Some("ashita.polplugins.args"))
        .set("pivot", profile_filename);

    // Update/create the profile ashita ini file
    fs::create_dir_all(ini_file_path.parent().unwrap()).await?;
    ashita_ini.write_to_file(&ini_file_path).with_context(|| {
        anyhow!(
            "Could not write Ashita ini file to {}",
            ini_file_path.display()
        )
    })?;

    Ok(())
}

pub async fn update_gamepad_config(profile: &Profile) -> anyhow::Result<()> {
    let ashita_directory = profile.install.try_get_ashita_dir()?;
    let ini_file_path = ashita_directory.join(format!(
        "config/boot/{}.ini",
        &profile.get_profile_filename()
    ));

    if !ini_file_path.exists() {
        return Ok(());
    }

    // Update profile ini file if it exists already
    let mut ashita_ini = ini::Ini::load_from_file(&ini_file_path).with_context(|| {
        anyhow!(
            "Could not load Ashita profile ini file: {}",
            ini_file_path.display()
        )
    })?;

    // Retrieve gamepad settings from registry
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let ffxi_reg = hklm.open_subkey("SOFTWARE\\WOW6432Node\\PlayOnline\\SQUARE\\FinalFantasyXI")?;

    ashita_ini
        .with_section(Some("ffxi.registry"))
        .set(
            "padmode000",
            ffxi_reg.get_value("padmode000").unwrap_or("-1".to_string()),
        )
        .set(
            "padsin000",
            ffxi_reg.get_value("padsin000").unwrap_or("-1".to_string()),
        )
        .set(
            "padguid000",
            ffxi_reg.get_value("padguid000").unwrap_or("-1".to_string()),
        );

    ashita_ini.write_to_file(&ini_file_path).with_context(|| {
        anyhow!(
            "Could not write Ashita ini file to {}",
            ini_file_path.display()
        )
    })?;

    Ok(())
}

async fn generate_pivot_ini(
    profile: &Profile,
    profile_filename: &str,
    server_folder_name: &str,
) -> anyhow::Result<()> {
    let ashita_dir = profile.install.try_get_ashita_dir()?;

    let dats_folder_path = ashita_dir.join("polplugins/DATs");

    let mut pivot_ini = ini::Ini::new();
    pivot_ini
        .with_section(Some("settings"))
        .set("root_path", dats_folder_path.to_str().unwrap())
        .set("debug_log", "false")
        .set("redirect_fopens", "true");

    let mut overlays = pivot_ini.with_section(Some("overlays"));
    let mut overlay_count = 0;

    // Add server-provided pivot first, if it exists
    let server_dat_folder_path = dats_folder_path.join(&server_folder_name);

    if server_dat_folder_path.exists() {
        overlays.set(overlay_count.to_string(), server_folder_name);
        overlay_count += 1;
    }

    // Additional pivots afterwards
    for pivot in &profile.extra_pivots {
        overlays.set(overlay_count.to_string(), pivot);
        overlay_count += 1;
    }

    // Create the profile pivot ini file
    let ini_file_path = profile
        .install
        .try_get_ashita_dir()?
        .join(format!("config/pivot/{}.ini", &profile_filename));

    fs::create_dir_all(ini_file_path.parent().unwrap()).await?;
    pivot_ini
        .write_to_file(&ini_file_path)
        .with_context(|| format!("Could not create file pivot at {}", ini_file_path.display()))?;

    Ok(())
}

async fn make_script_file(profile: &Profile, profile_filename: &str) -> anyhow::Result<()> {
    let script_name = format!("scripts/{}.txt", profile_filename);
    let script_path = profile.install.try_get_ashita_dir()?.join(&script_name);

    if script_path.exists() {
        let update_result = update_script_file(profile, &script_path).await;
        if update_result.is_err() {
            new_script_file(profile, &script_path).await
        } else {
            update_result
        }
    } else {
        new_script_file(profile, &script_path).await
    }
}

const XIL_START: &'static str = "\n## XI_LAUNCHER START";
const XIL_END: &'static str = "\n## XI_LAUNCHER END";

async fn new_script_file(profile: &Profile, script_path: &PathBuf) -> anyhow::Result<()> {
    fs::create_dir_all(script_path.parent().unwrap()).await?;
    let mut file = File::create(&script_path)
        .await
        .with_context(|| format!("Could not create file at {}", script_path.display()))?;

    file.write(
        r#"
/load thirdparty
/load addons
/load screenshot
"#
        .as_bytes(),
    )
    .await?;

    // Wrap launcher-managed lines in start and end markers
    file.write(XIL_START.as_bytes()).await?;
    file.write(b"\n").await?;
    if let Some(plugins) = &profile.enabled_plugins {
        for plugin in plugins {
            file.write(format!("/load {}\n", plugin).as_bytes()).await?;
        }
    }

    if let Some(addons) = &profile.enabled_addons {
        for addon in addons {
            file.write(format!("/addon load {}\n", addon).as_bytes())
                .await?;
        }
    }
    file.write(XIL_END.as_bytes()).await?;

    file.write(
        r#"

/bind insert /ashita
/bind SYSRQ /screenshot hide
/bind ^v /paste
/bind F11 /ambient
/bind F12 /fps
/bind ^F1 /ta <a10>
/bind ^F2 /ta <a11>
/bind ^F3 /ta <a12>
/bind ^F4 /ta <a13>
/bind ^F5 /ta <a14>
/bind ^F6 /ta <a15>
/bind !F1 /ta <a20>
/bind !F2 /ta <a21>
/bind !F3 /ta <a22>
/bind !F4 /ta <a23>
/bind !F5 /ta <a24>
/bind !F6 /ta <a25>

/wait 3
/ambient 255 255 255
"#
        .as_bytes(),
    )
    .await?;

    file.flush().await?;

    Ok(())
}

async fn update_script_file(profile: &Profile, script_path: &PathBuf) -> anyhow::Result<()> {
    let mut existing_file = File::open(&script_path)
        .await
        .with_context(|| format!("Could not create file at {}", script_path.display()))?;

    let mut content: String = String::new();
    existing_file.read_to_string(&mut content).await?;

    let start = content.find(XIL_START).ok_or(anyhow!(
        "Could not find '{XIL_START}' marker in {}",
        script_path.display()
    ))?;

    let end = (&content[start..]).find(XIL_END).ok_or(anyhow!(
        "Could not find '{XIL_END}' marker in {}",
        script_path.display()
    ))? + start
        + XIL_END.len();

    // Generate new launcher content
    let mut new_xil_content = BufWriter::new(Vec::new());
    new_xil_content.write(XIL_START.as_bytes()).await?;
    new_xil_content.write(b"\n").await?;
    if let Some(plugins) = &profile.enabled_plugins {
        for plugin in plugins {
            new_xil_content
                .write(format!("/load {}\n", plugin).as_bytes())
                .await?;
        }
    }

    if let Some(addons) = &profile.enabled_addons {
        for addon in addons {
            new_xil_content
                .write(format!("/addon load {}\n", addon).as_bytes())
                .await?;
        }
    }
    new_xil_content.write(XIL_END.as_bytes()).await?;

    let mut file = File::create(&script_path)
        .await
        .with_context(|| format!("Could not create file at {}", script_path.display()))?;

    file.write_all(&content.as_bytes()[..start]).await?;
    file.write_all(new_xil_content.buffer()).await?;
    file.write_all(&content.as_bytes()[end..]).await?;

    Ok(())
}
