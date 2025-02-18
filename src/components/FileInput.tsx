import { JSX } from "solid-js";
import { promptFolder } from "../util";

export interface FileInputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  onFileChange: (path: string | null) => any;
}

const FileInput = (props: FileInputProps) => {
  return (
    <input
      type="text"
      class="cursor-pointer"
      readOnly={true}
      onMouseDown={e => {
        e.preventDefault();

        if (e.button == 0) {
          // Prompt folder on left click
          promptFolder(path => {
            props.onFileChange(path);
          });
        } else {
          // Else clear
          props.onFileChange(null);
        }
      }}
      {...props}
    >
    </input>
  );
};

export default FileInput;
