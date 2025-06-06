import { useNavigate, useParams } from "@solidjs/router";
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  Match,
  on,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
import { createStore, produce, unwrap } from "solid-js/store";
import { AuthKind, commands, Profile } from "../bindings";
import { useData } from "../store";
import { DEFAULT_ADDONS, DEFAULT_PLUGINS } from "../store/profiles";
import FileInput from "./FileInput";
import Modal from "./Modal";
import ResolutionInput from "./ResolutionInput";
import ToggleList from "./ToggleList";

const ProfileEditor = () => {
  const params = useParams();
  const id = params.id ? parseInt(params.id) : 0;

  const navigate = useNavigate();

  const [getShowAddons, setShowAddons] = createSignal<boolean>(false);
  const [getShowPlugins, setShowPlugins] = createSignal<boolean>(false);
  const [getConfirmDelete, setConfirmDelete] = createSignal<boolean>(false);

  const onKeyDown = (ev: KeyboardEvent) => {
    if (ev.key == "Escape") {
      if (getShowAddons() || getShowPlugins()) {
        return;
      }
      navigate("/");
    }
  };

  onMount(() => {
    window.addEventListener("keydown", onKeyDown);
  });

  onCleanup(() => {
    window.removeEventListener("keydown", onKeyDown);
  });

  const { saveProfile, deleteProfile, getProfileInfo, nextProfileId } = useData();

  const [profile, updateProfileInfo] = createStore<Profile>(getProfileInfo(id));

  const getAshitaDirectory = createMemo(() => {
    return profile.install?.ashita_directory ?? (profile.install?.directory + "/Ashita") ?? "";
  });

  const [addons, { refetch: refetchAddons }] = createResource(async () => {
    let options = await commands.listAshitaAddons(getAshitaDirectory());
    if (options.status == "error") {
      return [];
    }

    const enabled = new Set(profile.enabled_addons ?? DEFAULT_ADDONS);
    return options.data.map(name => ({ name, selected: enabled.has(name) }));
  });

  const [plugins, { refetch: refetchPlugins }] = createResource(async () => {
    let options = await commands.listAshitaPlugins(getAshitaDirectory());
    if (options.status == "error") {
      return [];
    }

    const enabled = new Set(profile.enabled_plugins ?? DEFAULT_PLUGINS);
    return options.data.map(name => ({ name, selected: enabled.has(name) }));
  });

  createEffect(on(getAshitaDirectory, () => {
    refetchAddons();
    refetchPlugins();
  }));

  return (
    <div class="w-full h-full form">
      <div class="flex flex-wrap">
        <div class="field half pr-1">
          <label class="label" for="name">
            Profile name
          </label>
          <input
            class="input"
            id="name"
            type="text"
            placeholder={`Profile ${nextProfileId()}`}
            value={profile.name ?? ""}
            onInput={e => updateProfileInfo("name", e.target.value.trim())}
          >
          </input>
        </div>
        <div class="field half pl-1">
          <div class="half">
            <input
              id="windower"
              type="checkbox"
              checked={profile.use_windower}
              onChange={e => {
                updateProfileInfo("use_windower", e.target.checked);
                if (e.target.checked && !profile.install?.directory) {
                  updateProfileInfo(
                    "install",
                    produce(install => {
                      install = install || {};
                      install.directory = "C:\\Program Files (x86)\\PlayOnline\\SquareEnix";
                    }),
                  );
                }
              }}
            >
            </input>
            <label class="cursor-pointer" for="windower">Windower</label>
          </div>
          <div class="half">
            <input
              id="retail"
              type="checkbox"
              checked={profile.is_retail}
              onChange={e => {
                updateProfileInfo("is_retail", e.target.checked);
                if (e.target.checked && !profile.install?.directory) {
                  updateProfileInfo(
                    "install",
                    produce(install => {
                      install = install || {};
                      install.directory = "C:\\Program Files (x86)\\PlayOnline\\SquareEnix";
                    }),
                  );
                }
              }}
            >
            </input>
            <label class="cursor-pointer" for="retail">Retail</label>
          </div>
        </div>

        <div class="field half">
          <label class="label" for="install">
            Game directory
          </label>
          <FileInput
            id="install"
            placeholder="C:\Games\PrivateFFXI"
            value={profile.install?.directory ?? ""}
            onFileChange={path => {
              updateProfileInfo(
                "install",
                produce(install => {
                  install = install || {};
                  install.directory = path;
                }),
              );
            }}
          >
          </FileInput>
        </div>

        <Switch>
          <Match when={!profile.use_windower}>
            <div class="field half">
              <label class="label" for="ashita">
                Ashita v4 directory
              </label>
              <FileInput
                id="ashita"
                placeholder="Optional, if it exists in the Game Directory"
                value={profile.install?.ashita_directory ?? ""}
                onFileChange={path => {
                  updateProfileInfo(
                    "install",
                    produce(install => {
                      install = install || {};
                      install.ashita_directory = path;
                    }),
                  );
                }}
              >
              </FileInput>
            </div>
          </Match>

          <Match when={profile.use_windower}>
            <div class="field half">
              <label class="label" for="windower">
                Windower directory
              </label>
              <FileInput
                id="windower"
                placeholder="C:\Tools\Windower"
                value={profile.install?.windower_directory ?? ""}
                onFileChange={path => {
                  updateProfileInfo(
                    "install",
                    produce(install => {
                      install = install || {};
                      install.windower_directory = path;
                    }),
                  );
                }}
              >
              </FileInput>
            </div>
          </Match>
        </Switch>

        <Show when={!profile.is_retail}>
          <div class="field half">
            <label class="label" for="server">
              Server address
            </label>
            <input
              id="server"
              type="text"
              placeholder="localhost"
              value={profile.server ?? ""}
              onInput={e => updateProfileInfo("server", e.target.value.trim())}
            >
            </input>
          </div>

          <div class="field half">
            <div class="half">
              <input
                id="manual_auth"
                type="checkbox"
                checked={profile.manual_auth ?? false}
                onChange={e => updateProfileInfo("manual_auth", e.target.checked)}
              >
              </input>
              <label for="manual_auth">Manual login</label>
            </div>
            <div class="half">
              <input
                id="hairpin"
                type="checkbox"
                checked={profile.hairpin ?? false}
                onChange={e => updateProfileInfo("hairpin", e.target.checked)}
              >
              </input>
              <label for="hairpin">Hairpin</label>
            </div>
          </div>

          <Show when={!profile.manual_auth}>
            <div class="field half">
              <label class="label" for="account_name">
                Account name
              </label>
              <input
                id="account_name"
                type="text"
                placeholder="Account name"
                value={profile.account_name ?? ""}
                onInput={e => updateProfileInfo("account_name", e.target.value.trim())}
              >
              </input>
            </div>

            <div class="field half">
              <label class="label" for="auth_kind">
                Login method
              </label>
              <select
                id="auth_kind"
                onChange={e => {
                  const kind = e.target.value as AuthKind;
                  updateProfileInfo("auth_kind", kind);
                  if (kind != "Password" && profile.password) {
                    updateProfileInfo("password", null);
                  }
                }}
              >
                <option
                  value={"Token"}
                  selected={profile.auth_kind == "Token"}
                >
                  Stay logged in with token (if supported)
                </option>
                <option
                  value={"ManualPassword"}
                  selected={profile.auth_kind == "ManualPassword"}
                >
                  Prompt for password each login
                </option>
                <option
                  value={"Password"}
                  selected={profile.auth_kind == "Password"}
                >
                  Store password in plaintext
                </option>
              </select>
            </div>

            <Show when={profile.auth_kind == "Password"}>
              <div class="field half">
                <label class="label" for="password">
                  Password
                </label>
                <input
                  id="password"
                  type="password"
                  placeholder="This will be stored in plaintext"
                  value={profile.password ?? ""}
                  onInput={e => updateProfileInfo("password", e.target.value == "" ? null : e.target.value)}
                >
                </input>
              </div>
              <div class="field half"></div>
            </Show>
          </Show>
        </Show>

        <Show when={!profile.use_windower}>
          <div class="field half">
            <button
              class="button neutral w-full"
              onClick={() => setShowAddons(true)}
            >
              Addons
            </button>
          </div>
          <div class="field half">
            <button
              class="button neutral w-full"
              onClick={() => setShowPlugins(true)}
            >
              Plugins
            </button>
          </div>

          <div class="field half">
            <ResolutionInput
              label="Resolution"
              initial={profile.resolution
                ? { width: profile.resolution.width, height: profile.resolution.height }
                : undefined}
              onChange={(width, height) => {
                updateProfileInfo("resolution", { width, height });
              }}
            />
          </div>
          <div class="field half">
            <ResolutionInput
              label="Background resolution"
              initial={profile.background_resolution
                ? { width: profile.background_resolution.width, height: profile.background_resolution.height }
                : undefined}
              onChange={(width, height) => {
                updateProfileInfo("background_resolution", { width, height });
              }}
            />
          </div>

          <div class="field half">
            <ResolutionInput
              label="Menu resolution"
              initial={profile.menu_resolution
                ? { width: profile.menu_resolution.width, height: profile.menu_resolution.height }
                : undefined}
              onChange={(width, height) => {
                updateProfileInfo("menu_resolution", { width, height });
              }}
            />
          </div>

          <div class="field half">
            <label class="label">
              Window position (centered is -1)
            </label>
            <div class="w-full flex flex-row gap-1 items-center">
              <div class="w-1/2 relative inline-flex items-center">
                <span class="absolute left-3 text-gray-400">x:</span>
                <input
                  type="number"
                  step={10}
                  value={profile.start_pos_x ?? -1}
                  onChange={e => {
                    let val = parseInt(e.target.value);
                    if (isNaN(val)) {
                      val = -1;
                    }
                    updateProfileInfo("start_pos_x", val);
                  }}
                  class="text-center"
                >
                </input>
              </div>
              <div class="w-1/2 relative inline-flex items-center">
                <span class="absolute left-3 text-gray-400">y:</span>
                <input
                  step={10}
                  type="number"
                  value={profile.start_pos_y ?? -1}
                  onChange={e => {
                    let val = parseInt(e.target.value);
                    if (isNaN(val)) {
                      val = -1;
                    }
                    updateProfileInfo("start_pos_y", val);
                  }}
                  class="text-center"
                >
                </input>
              </div>
            </div>
          </div>

          <div class="field half">
            <input
              id="enable_gamepad"
              type="checkbox"
              checked={profile.enable_gamepad ?? false}
              onChange={e => updateProfileInfo("enable_gamepad", e.target.checked)}
            >
            </input>
            <label for="enable_gamepad">Gamepad</label>
          </div>
          <div class="field half"></div>
          <Show when={profile.enable_gamepad}>
            <div class="field">
              <div class="half">
                <button
                  class="button neutral w-full"
                  disabled={!profile.install?.directory}
                  onClick={async () => await commands.configureGamepad(profile.install!.directory!)}
                >
                  Configure Gamepad
                </button>
              </div>
              <div class="half">
                <input
                  id="enable_gamepad_background"
                  type="checkbox"
                  checked={profile.enable_gamepad_background ?? false}
                  onChange={e => updateProfileInfo("enable_gamepad_background", e.target.checked)}
                >
                </input>
                <label for="enable_gamepad_background">Allow gamepad use in background</label>
              </div>
            </div>
          </Show>
        </Show>

        <Show when={profile.use_windower}>
          <div class="field half">
            <label class="label" for="windower_profile">
              Windower profile name
            </label>
            <input
              id="windower_profile"
              type="text"
              placeholder="Profile name"
              value={profile.windower_profile ?? ""}
              onInput={e => updateProfileInfo("windower_profile", e.target.value.trim())}
            >
            </input>
          </div>
          <div class="field half inline-flex items-center pt-5">
            Ensure the Windower profile exists before launching.
          </div>

          <div class="field">
            If the server requires pivoting of DATs/sounds, setup XIPivot for Windower and add the overlay for it:{" "}
            <code>{profile.server}</code>
          </div>
        </Show>

        <div class="w-full flex flex-row space-x-4">
          <button
            class="button accept w-full"
            onClick={async () => {
              await saveProfile(id, unwrap(profile));
              navigate("/");
            }}
          >
            Save
          </button>

          <button
            class="button decline w-full"
            onClick={async () => {
              if (id) {
                if (getConfirmDelete()) {
                  await deleteProfile(id);
                  setConfirmDelete(false);
                  navigate("/");
                } else {
                  setConfirmDelete(true);
                }
              } else {
                navigate("/");
              }
            }}
          >
            {getConfirmDelete() ? "Press again to confirm deletion" : "Delete"}
          </button>
        </div>

        <button
          class="button w-full mt-2"
          onClick={async () => {
            navigate("/");
          }}
        >
          Back
        </button>
      </div>

      <Modal when={getShowAddons()} close={() => setShowAddons(false)}>
        <ToggleList
          options={addons() ?? []}
          onComplete={selected => {
            updateProfileInfo("enabled_addons", selected);
            setShowAddons(false);
          }}
        >
        </ToggleList>
      </Modal>
      <Modal when={getShowPlugins()} close={() => setShowPlugins(false)}>
        <ToggleList
          options={plugins() ?? []}
          onComplete={selected => {
            updateProfileInfo("enabled_plugins", selected);
            setShowPlugins(false);
          }}
        >
        </ToggleList>
      </Modal>
    </div>
  );
};

export default ProfileEditor;
