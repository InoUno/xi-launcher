import { open } from "@tauri-apps/plugin-dialog";
import toast from "solid-toast";
import { Result } from "./bindings";

export async function promptFolder(
  setFolder: (path: string | null) => any,
  defaultPath?: string | null,
) {
  const selected = (await open({
    multiple: false,
    directory: true,
    defaultPath: defaultPath ?? undefined,
  })) as string | null;

  if (selected) {
    setFolder(selected);
  }
}

export function unwrapResult<T, E>(result: Result<T, E>): T {
  if (result.status == "error") {
    toast.error(result.error as string);
    console.error(result.error);
    throw result.error;
  }
  return result.data;
}

const KILOBYTES = 1024;
const MEGABYTES = KILOBYTES * 1024;
const GIGABYTES = MEGABYTES * 1024;

export function bytesToReadable(bytes: number, scaleBytes?: number): string {
  scaleBytes = scaleBytes ?? bytes;

  if (scaleBytes > GIGABYTES) {
    return `${(bytes / GIGABYTES).toFixed(1)} GB`;
  } else if (scaleBytes > MEGABYTES) {
    return `${(bytes / MEGABYTES).toFixed(1)} MB`;
  } else {
    return `${(bytes / KILOBYTES).toFixed(1)} KB`;
  }
}
