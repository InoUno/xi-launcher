import { HiSolidFolderOpen } from "solid-icons/hi";
import { JSX } from "solid-js";
import { promptFolder } from "../util";

export interface FileInputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  onFileChange: (path: string | null) => any;
  defaultPath?: string;
}

const FileInput = (props: FileInputProps) => {
  return (
    <div class="relative inline-flex items-center w-full">
      <input
        type="text"
        onInput={e => {
          props.onFileChange(e.target.value);
        }}
        {...props}
      >
      </input>
      <button
        class="cursor-pointer absolute right-2 p-2 hover:text-yellow-200"
        onMouseDown={e => {
          e.preventDefault();

          if (e.button == 0) {
            // Prompt folder on left click
            promptFolder(path => {
              props.onFileChange(path);
            }, props.defaultPath);
          } else {
            // Else clear
            props.onFileChange(null);
          }
        }}
      >
        <HiSolidFolderOpen size={24} />
      </button>
    </div>
  );
};

export default FileInput;
