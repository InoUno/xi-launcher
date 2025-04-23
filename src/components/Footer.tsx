import { createResource, createSignal, Match, Switch } from "solid-js";
import { checkForUpdate, startUpdate } from "../updater";

const Footer = () => {
  const [update] = createResource(checkForUpdate);

  const [getProgress, setProgress] = createSignal<string>("");

  return (
    <Switch>
      <Match when={getProgress()}>
        <div class="p-2 w-full flex flex-grow items-center justify-end">
          Updating {getProgress()}
        </div>
      </Match>
      <Match when={!getProgress() && !update.loading && update()}>
        <div class="p-2 w-full flex flex-grow items-center justify-end">
          <div class="pr-3">An update is available for XI Launcher ({update()?.version})</div>
          <div>
            <button
              class="button accept w-full"
              onClick={async () => {
                await startUpdate(update()!, setProgress);
              }}
            >
              Update
            </button>
          </div>
        </div>
      </Match>
    </Switch>
  );
};

export default Footer;
