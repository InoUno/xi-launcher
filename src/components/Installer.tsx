import { Channel } from "@tauri-apps/api/core";
import { createMemo, createSignal, onCleanup, Show } from "solid-js";
import { createStore, produce } from "solid-js/store";
import toast from "solid-toast";
import { commands, FileInstallConfig, InstallTaskProgress } from "../bindings";
import { unwrapResult } from "../util";

interface InstallStatus {
  state?: string;
  unknown_progress?: boolean;
  current_progress?: number;
  expected_final?: number;
}

interface InstallerProps {
  id: number;
  downloadInfo: FileInstallConfig[];
  isComplete: () => any;
}

const KILOBYTES = 1024;
const MEGABYTES = KILOBYTES * 1024;
const GIGABYTES = MEGABYTES * 1024;

const Installer = (props: InstallerProps) => {
  const [getIsInstalling, setIsInstalling] = createSignal<boolean>(false);
  const [status, setStatus] = createStore<InstallStatus>({});

  const startInstall = async () => {
    if (getIsInstalling()) {
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

    let result = await commands.installGameForProfile(props.id, props.downloadInfo, channel);
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
    if (status.expected_final > GIGABYTES) {
      return `${(status.current_progress / GIGABYTES).toFixed(1)} GB`;
    } else if (status.expected_final > MEGABYTES) {
      return `${(status.current_progress / MEGABYTES).toFixed(1)} MB`;
    } else {
      return `${(status.current_progress / KILOBYTES).toFixed(1)} KB`;
    }
  });

  const sizeTotal = createMemo(() => {
    if (!status.expected_final) {
      return null;
    }
    if (status.expected_final > GIGABYTES) {
      return `${(status.expected_final / GIGABYTES).toFixed(1)} GB`;
    } else if (status.expected_final > MEGABYTES) {
      return `${(status.expected_final / MEGABYTES).toFixed(1)} MB`;
    } else {
      return `${(status.expected_final / KILOBYTES).toFixed(1)} KB`;
    }
  });

  onCleanup(() => {
    commands.cancelPossibleProfileTask(props.id);
  });

  return (
    <div class="flex flex-col w-full items-center">
      <Show when={!getIsInstalling()}>
        <div>The game can be downloaded and installed from the server.</div>
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
