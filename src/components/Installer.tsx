import { Channel } from "@tauri-apps/api/core";
import { createMemo, createSignal, onCleanup, Show } from "solid-js";
import { createStore, produce } from "solid-js/store";
import toast from "solid-toast";
import { commands, FileInstallConfig, InstallTaskProgress, Profile } from "../bindings";
import { bytesToReadable, unwrapResult } from "../util";

interface InstallStatus {
  state?: string;
  unknown_progress?: boolean;
  current_progress?: number;
  expected_final?: number;
}

interface InstallerProps {
  profile: Profile;
  downloadInfo: FileInstallConfig[];
  isComplete: () => any;
}

const foldersThatNeedSubFolder = new Set(
  ["Program Files", "Program Files (x86)"],
);

const Installer = (props: InstallerProps) => {
  const [getIsInstalling, setIsInstalling] = createSignal<boolean>(false);
  const [status, setStatus] = createStore<InstallStatus>({});
  const [getFolderName, setFolderName] = createSignal<string>("");

  const requiresFolderName = createMemo(() => {
    const directory = props.profile.install?.directory;
    if (!directory) {
      return true;
    }
    const split = directory.split("\\");
    let idx = split.length - 1;
    let last = split[idx];
    while (last.trim().length == 0) {
      if (idx <= 0) {
        return true;
      }
      idx--;
      last = split[idx];
    }

    if (last.indexOf(":") > -1) {
      return true;
    }
    return foldersThatNeedSubFolder.has(last);
  });

  const startInstall = async () => {
    if (getIsInstalling()) {
      return;
    }

    const folderName = getFolderName();
    if (folderName && props.profile.install) {
      let { profile } = props;
      if (!profile.install!.directory?.endsWith("\\")) {
        profile.install!.directory += "\\";
      }
      profile.install!.directory += folderName;
      await commands.saveProfile(profile.id, profile);
      console.log(`Updated install directory to: ${profile.install!.directory}`);
    } else if (requiresFolderName()) {
      toast.error("Please provide a folder name to install the game into.");
      return;
    }

    setIsInstalling(true);

    const channel = new Channel<InstallTaskProgress>();
    channel.onmessage = (message: InstallTaskProgress) => {
      console.log(message);
      switch (message.event) {
        case "Pending":
          setStatus({
            state: "Pending",
            current_progress: 0,
            expected_final: 1,
          });
          break;
        case "DownloadStarted":
          setStatus(produce(status => {
            status.state = "Downloading";
            status.current_progress = 0;
            status.expected_final = message.data.content_length;
          }));
          break;
        case "DownloadProgress":
          setStatus(produce(status => {
            status.current_progress = message.data.finished_length;
          }));
          break;
        case "Installing":
          setStatus(produce(status => {
            status.state = "Installing";
            status.unknown_progress = true;
          }));
          break;
        case "Error":
          toast.error(`Failed to install: ${message.data}`);
          setStatus({});
          props.isComplete();
          break;
        case "Complete":
          setStatus({});
          props.isComplete();
          break;
      }
    };

    let result = await commands.installGameForProfile(props.profile.id, props.downloadInfo, channel);
    if (result.status == "error") {
      props.isComplete();
    }
    unwrapResult(result);
  };

  const percentageComplete = createMemo(() => {
    return Math.floor((status.current_progress ?? 0) / (status.expected_final ?? 1) * 100);
  });

  const sizeProgress = createMemo(() => {
    if (!status.expected_final || !status.current_progress) {
      return null;
    }
    return bytesToReadable(status.current_progress, status.expected_final);
  });

  const sizeTotal = createMemo(() => {
    if (!status.expected_final) {
      return null;
    }
    return bytesToReadable(status.expected_final);
  });

  onCleanup(() => {
    commands.cancelPossibleProfileTask(props.profile.id);
  });

  return (
    <div class="flex flex-col w-full items-center gap-2">
      <Show when={!getIsInstalling()}>
        <div>The client can be downloaded and installed from the server.</div>
        <Show
          fallback={<div>Please provide a folder name the game should be installed into:</div>}
          when={!requiresFolderName()}
        >
          <div>
            Do you want to place the game files directly into{" "}
            <code class="whitespace-nowrap text-green-200">{props.profile.install?.directory}</code>?
          </div>
          <div>If not, please provide a folder name that the game files will be installed into:</div>
        </Show>
        <div class="form field inline-flex items-center">
          <code class="whitespace-nowrap">
            {props.profile.install?.directory}
            {props.profile.install?.directory?.endsWith("\\") ? "" : "\\"}
          </code>
          <input
            type="text"
            value={getFolderName()}
            onInput={e => setFolderName(e.target.value)}
            placeholder="Folder name"
          >
          </input>
        </div>
        <button
          class="button accept"
          onClick={startInstall}
          disabled={getIsInstalling()}
        >
          Download and install
        </button>
      </Show>

      <Show when={status.state !== undefined}>
        <div>
          {status.state}
        </div>
        <div class="border border-green-600 rounded-xl h-10 relative w-full">
          <Show
            when={!status.unknown_progress}
            fallback={<div class="bg-green-600 rounded-xl h-full w-full loader"></div>}
          >
            <div
              class="bg-green-600 rounded-xl px-5 h-full  w-full flex items-center font-bold"
              style={{ width: percentageComplete() + "%" }}
            >
              {percentageComplete()}%
            </div>
          </Show>
        </div>
        <div class="px-5 w-full flex items-end font-bold">
          <Show when={sizeTotal()} fallback={<span>&nbsp;</span>}>
            {sizeProgress()} / {sizeTotal()}
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default Installer;
