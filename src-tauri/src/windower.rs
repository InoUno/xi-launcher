use std::{fs::File, io::Write, os::windows::process::CommandExt, process::Command};

use anyhow::{anyhow, Context};

use edit_xml::{Document, Element};

use crate::config::profiles::{AuthKind, Profile};

pub async fn launch_game(
    profile: &Profile,
    provided_password: Option<String>,
) -> anyhow::Result<()> {
    let mut exe = profile.install.try_get_windower_dir()?;
    exe.push("Windower.exe");

    if !exe.exists() {
        return Err(anyhow!(
            "Windower executable not found at: {}",
            exe.display()
        ));
    }

    let Some(windower_profile) = &profile.windower_profile else {
        return Err(anyhow!("Missing Windower profile name"));
    };

    update_windower_profile(profile, &windower_profile, provided_password)?;

    let working_dir = exe.parent().unwrap().to_path_buf();

    let mut cmd = Command::new("cmd");
    cmd.creation_flags(0x00000008) // DETACHED_PROCESS
        .current_dir(working_dir)
        .arg("/C")
        .arg(exe)
        .arg("-p")
        .arg(windower_profile);
    // .arg(format!("-p=\"{}\"", windower_profile));

    tracing::info!(
        "Launching Windower with profile: {}",
        profile.name.as_ref().unwrap_or(&"unnamed".to_string())
    );
    tracing::info!("Command: {:#?}", cmd);

    let mut _child = cmd.spawn().context("Failed to start Windower")?;

    tauri::async_runtime::spawn(async move {
        if let Ok(exit_code) = _child.wait() {
            tracing::debug!("Got exit code: {exit_code}");
        }
    });

    Ok(())
}

pub fn update_windower_profile(
    profile: &Profile,
    profile_name: &str,
    provided_password: Option<String>,
) -> anyhow::Result<()> {
    // Find matching profile in Windower settings.xml
    let settings_path = profile.install.try_get_windower_dir()?.join("settings.xml");
    let mut needs_update = false;
    let mut doc = Document::parse_file(&settings_path)?;
    let xml_profile = locate_profile_with_name(&doc, profile_name).ok_or(anyhow!(
        "Could not find Windower profile called '{profile_name}'. Make sure it exists in Windower before launching."
    ))?;

    // Prepare args for boot loader
    let mut args = vec![];
    if !profile.manual_auth && !profile.is_retail {
        if let Some(server) = &profile.server {
            args.push(format!("--server {server}"));
        }

        if profile.hairpin {
            args.push("--hairpin".to_string());
        }

        let account_name = profile
            .account_name
            .as_ref()
            .cloned()
            .ok_or(anyhow!("Missing username."))?;

        match profile.auth_kind {
            AuthKind::Token => {
                if let Some(password) = provided_password {
                    args.push(format!("--user {account_name} --pass {password}"));
                }

                args.push(format!(
                    "--tokenfile {}",
                    profile
                        .get_token_path()
                        .ok_or_else(|| anyhow!("Missing token path."))?
                        .to_str()
                        .unwrap(),
                ));
            }
            AuthKind::Password => {
                let Some(stored_password) = &profile.password else {
                    return Err(anyhow!("No password stored locally."));
                };

                args.push(format!("--user {account_name} --pass {stored_password}"));
            }
            AuthKind::ManualPassword => {
                let Some(password) = provided_password else {
                    return Err(anyhow!("Expected a provided password."));
                };

                args.push(format!("--user {account_name} --pass {password}"));
            }
        }
    }

    // Update args for profile if necessary
    let args_text = args.join(" ");

    if let Some(xml_args) = xml_profile.find(&doc, "args") {
        if xml_args.text_content(&doc) != args_text {
            xml_args.set_text_content(&mut doc, args_text);
            needs_update = true;
        }
    } else {
        Element::build("args")
            .add_text(args_text)
            .push_to(&mut doc, xml_profile);
        needs_update = true;
    }

    // Update bootloader path if necessary
    let bootloader_path = profile
        .get_bootloader_path()
        .ok_or(anyhow!("Could not determine Windower bootloader path"))?
        .join("xiloader.exe");

    let bootloader_str = bootloader_path.as_os_str().to_str().unwrap_or_default();

    if let Some(xml_executable) = xml_profile.find(&doc, "executable") {
        if xml_executable.text_content(&doc) != bootloader_str {
            xml_executable.set_text_content(&mut doc, bootloader_str);
            needs_update = true;
        }
    } else {
        Element::build("executable")
            .add_text(bootloader_str)
            .push_to(&mut doc, xml_profile);
        needs_update = true;
    }

    if needs_update {
        tracing::info!("About to update Windower settings.xml");
        let mut settings_file = File::create(&settings_path)?;
        settings_file.write_all(doc.write_str()?.as_bytes())?;
        settings_file.flush()?;
        tracing::info!("Updated Windower settings.xml");
    }

    Ok(())
}

fn locate_profile_with_name(doc: &Document, name: &str) -> Option<Element> {
    let container = doc.container();
    let settings = container.find(&doc, "settings")?;
    let profiles = settings.find_all(&doc, "profile");
    for profile in profiles {
        if profile.attribute(&doc, "name") == Some(name) {
            return Some(profile);
        }
    }
    None
}
