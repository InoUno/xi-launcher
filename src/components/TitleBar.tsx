import { getCurrentWindow } from "@tauri-apps/api/window";
import "./TitleBar.css";
import { FaSolidWindowMinimize, FaSolidXmark } from "solid-icons/fa";

const TitleBar = () => {
  const appWindow = getCurrentWindow();
  return (
    <div data-tauri-drag-region class="titlebar">
      <div class="px-2 py-1">
        XI Launcher
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
