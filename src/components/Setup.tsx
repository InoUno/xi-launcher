import { Channel } from "@tauri-apps/api/core";
import { createSignal, Show } from "solid-js";
import { commands } from "../bindings";
import { useData } from "../store";
import { promptFolder } from "../util";
import Modal from "./Modal";

const Setup = () => {
  const { getInstallDir, getNeedsInstall, updateInstallDir } = useData();
  const [getModalOpen, setModalOpen] = createSignal<boolean>(false);
  const [getDownloadLink, setDownloadLink] = createSignal<string>(
    "magnet:?xt=urn:btih:abb9d35cfad710117ee642db402b371fb2c70cf4&dn=xi_install_mini.7z&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fexplodie.org%3A6969&tr=udp%3A%2F%2Ftracker.empire-js.us%3A1337",
  );

  const startInstall = async () => {
    // try {
    //   const channel = new Channel<InstallMessage>();

    //   channel.onmessage = message => {
    //     console.log("download", message);
    //   };

    //   const res = await commands.installFromUrl(getDownloadLink(), channel);

    //   if (res.status == "error") {
    //     console.error(res.error);
    //   }
    // } catch (err) {
    //   console.error(err);
    // }
  };

  return (
    <div>
      <h2>
        Setup
      </h2>

      <table class="form">
        <tbody>
          <tr>
            <td>
              Install directory:
            </td>
            <td>
              <div
                class="cursor-pointer"
                onClick={() => promptFolder(async path => await updateInstallDir(path))}
              >
                <Show
                  when={getInstallDir()}
                  fallback={
                    <span class="italic text-yellow-300 font-mono text-lg underline underline-offset-4">Not set</span>
                  }
                >
                  <span class="text-green-300 italic font-mono text-lg underline underline-offset-4">
                    {getInstallDir()}
                  </span>
                </Show>
              </div>
            </td>
          </tr>
        </tbody>
      </table>

      <Show when={getNeedsInstall()}>
        <div class="italic text-slate-300 my-2">
          Installation is required, since the game and/or Ashita is missing from the install directory.
        </div>

        <button
          class="button accept"
          onClick={() => setModalOpen(true)}
        >
          Install via link
        </button>

        <Modal when={getModalOpen()} close={() => setModalOpen(false)}>
          <div>
            <form
              class="form"
              onSubmit={e => {
                e.preventDefault();
                startInstall();
                setModalOpen(false);
              }}
            >
              <div class="flex flex-col">
                <input
                  type="text"
                  placeholder="Link to installer"
                  value={getDownloadLink()}
                  onChange={e => setDownloadLink(e.target.value)}
                >
                </input>
                <button class="button accept" type="submit">
                  Install
                </button>
              </div>
            </form>
          </div>
        </Modal>
      </Show>
    </div>
  );
};

export default Setup;
