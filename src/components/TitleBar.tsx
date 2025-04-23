import { getCurrentWindow } from "@tauri-apps/api/window";
import "./TitleBar.css";
import { getVersion } from "@tauri-apps/api/app";
import { FaSolidWindowMinimize, FaSolidXmark } from "solid-icons/fa";
import { createResource } from "solid-js";

const TitleBar = () => {
  const [appVersion] = createResource(getVersion);
  const appWindow = getCurrentWindow();

  return (
    <div data-tauri-drag-region class="titlebar">
      <div class="px-2 py-1">
        XI Launcher v{appVersion()}
      </div>
      <div class="titlebar-right">
        <div class="titlebar-button" id="titlebar-minimize" onClick={appWindow.minimize}>
          <FaSolidWindowMinimize></FaSolidWindowMinimize>
        </div>
        <div class="titlebar-button" id="titlebar-close" onClick={appWindow.close}>
          <FaSolidXmark></FaSolidXmark>
        </div>
      </div>
    </div>
  );
};

export default TitleBar;
