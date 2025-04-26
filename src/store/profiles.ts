import { createEffect, createResource } from "solid-js";
import { createStore } from "solid-js/store";
import { commands, Profile, Profiles } from "../bindings";
import { unwrapResult } from "../util";

export function createProfilesStore() {
  const [profiles, setProfiles] = createStore<Profiles>({});

  const [resourceProfiles, { refetch: profilesRefetch }] = createResource(async () =>
    unwrapResult(await commands.getProfiles())
  );

  // Initialize profile names
  createEffect(() => {
    setProfiles(resourceProfiles() || {});
  });

  // Profile update helper
  const saveProfile = async (id: number | null, profile?: Profile) => {
    try {
      await saveProfileWithDefaults(id == 0 ? null : id, profile ?? profiles.map?.[id!]);
      await profilesRefetch();
    } catch (err) {
      console.error(err);
    }
  };

  // Profile delete helper
  const deleteProfile = async (id: number) => {
    try {
      unwrapResult(await commands.deleteProfile(id));
      await profilesRefetch();
    } catch (err) {
      console.error(err);
    }
  };

  return {
    profiles,
    saveProfile,
    profilesRefetch,
    deleteProfile,
  };
}

export const DEFAULT_ADDONS: Set<string> = new Set([
  "aspect",
  "distance",
  "drawdistance",
  "fps",
  "instantah",
  "logs",
  "macrofix",
  "mipmap",
  "timestamp",
  "tparty",
]);

export const DEFAULT_PLUGINS: Set<string> = new Set([
  "addons",
  "thirdparty",
  "screenshot",
]);

export async function saveProfileWithDefaults(id: number | null, profile?: Profile) {
  if (!profile) {
    console.error("No profile to be saved");
    return;
  }

  if ((!profile.enabled_addons || !profile.enabled_plugins) && profile.install?.directory) {
    const ashita_directory = profile.install.ashita_directory ?? (profile.install.directory + "/Ashita");
    if (ashita_directory) {
      if (!profile.enabled_addons) {
        const addons = await commands.listAshitaAddons(ashita_directory);
        if (addons.status == "ok") {
          profile.enabled_addons = addons.data.filter(v => DEFAULT_ADDONS.has(v));
        }
      }

      if (!profile.enabled_plugins) {
        const plugins = await commands.listAshitaPlugins(ashita_directory);
        if (plugins.status == "ok") {
          profile.enabled_plugins = plugins.data.filter(v => DEFAULT_PLUGINS.has(v));
        }
      }
    }
  }

  if (!id) {
    if (!profile.server && !profile.is_retail) {
      profile.server = "localhost";
    }
  }

  unwrapResult(await commands.saveProfile(id, profile));
}
