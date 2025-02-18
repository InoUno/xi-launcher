import { Channel } from "@tauri-apps/api/core";
import { createMemo, createSignal, onCleanup, Show } from "solid-js";
import { createStore, produce } from "solid-js/store";
import toast from "solid-toast";
import { commands, UpdateTaskMessage } from "../bindings";
import { unwrapResult } from "../util";

interface UpdateStatus {
  id?: string;
  state?: string;
  unknown_progress?: boolean;
  current_progress?: number;
  expected_final?: number;
}

const Updater = (props: { id: number; isComplete: () => any; }) => {
  const [getIsUpdating, setIsUpdating] = createSignal<boolean>(false);
  const [status, setStatus] = createStore<UpdateStatus>({});

  const startUpdate = async () => {
    if (getIsUpdating()) {
      return;
    }
    setIsUpdating(true);

    const channel = new Channel<UpdateTaskMessage>();
    channel.onmessage = (message: UpdateTaskMessage) => {
      switch (message.event) {
        case "DownloadPending":
          setStatus({
            id: message.data.id,
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
        case "DownloadFinished":
          break;
        case "UnpackPending":
          setStatus(produce(status => {
            status.state = "Unpacking";
            status.unknown_progress = true;
          }));
          break;
        case "UnpackFinished":
          setStatus(produce(status => {
            status.state = "Unpacking";
            status.unknown_progress = false;
            status.current_progress = 1;
            status.expected_final = 1;
          }));
          break;
        case "FailedSpecific":
          toast.error(`Failed to update ${message.data.id}.`);
          setStatus({});
          break;
        case "Done":
          setStatus({});
          props.isComplete();
          break;
      }
    };

    unwrapResult(await commands.updateProfileServerFiles(props.id, channel));
  };

  const percentageComplete = createMemo(() => {
    return Math.floor((status.current_progress ?? 0) / (status.expected_final ?? 1) * 100);
  });

  onCleanup(() => {
    commands.cancelPossibleProfileTask(props.id);
  });

  return (
    <div class="flex flex-col w-full items-center">
      <div>An update is available.</div>

      <Show
        when={status.state !== undefined}
        fallback={
          <button
            class="button accept"
            onClick={startUpdate}
            disabled={getIsUpdating()}
          >
            Update
          </button>
        }
      >
        <div>
          [{status.id}] {status.state}
        </div>
        <div class="border border-green-600 rounded-xl h-10 relative  w-full">
          <Show
            when={!status.unknown_progress}
            fallback={<div class="bg-green-600 rounded-xl h-full  w-full loader"></div>}
          >
            <div
              class="bg-green-600 rounded-xl px-5 h-full  w-full flex items-center font-bold"
              style={{ width: percentageComplete() + "%" }}
            >
              {percentageComplete()}%
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default Updater;
