import { createEffect, createResource, createSignal } from "solid-js";
import { commands } from "../bindings";
import { unwrapResult } from "../util";

export function createSettingsStore() {
  const [getInstallDir, setInstallDir] = createSignal<string | null>();
  const [getNeedsInstall, setNeedsInstall] = createSignal<boolean>(false);

  // const [resourceInstallState, { refetch: refetchInstallState }] = createResource(async () =>
  //   unwrapResult(await commands.getInstallState())
  // );

  // Initialize store
  createEffect(() => {
    // const state = resourceInstallState();
    // if (state) {
    //   setInstallDir(state.install_dir);
    //   setNeedsInstall(state.needs_install);
    // }
  });

  // Update helpers
  const updateInstallDir = async (path: string | null) => {
    try {
      // unwrapResult(await commands.setInstallDir(path));
      // await refetchInstallState();
    } catch (err) {
      console.error(err);
    }
  };

  return {
    getInstallDir,
    getNeedsInstall,
    updateInstallDir,
  };
}
