import { FaSolidXmark } from "solid-icons/fa";
import { FlowProps, onCleanup, onMount, Show } from "solid-js";

interface ModalProps extends FlowProps {
  when: boolean;
  close: () => any;
}

const Modal = (props: ModalProps) => {
  const onKeyDown = (ev: KeyboardEvent) => {
    if (ev.key == "Escape") {
      props.close();
    }
  };

  onMount(() => {
    window.addEventListener("keydown", onKeyDown);
  });

  onCleanup(() => {
    window.removeEventListener("keydown", onKeyDown);
  });

  return (
    <Show when={props.when}>
      <div class="fixed top-0 left-0 w-screen h-full bg-slate-900 opacity-95"></div>
      <div class="absolute top-0 left-0 m-0 p-3 w-screen h-full">
        <div class="relative h-full w-6/7 px-8 pb-4 pt-10 rounded-xl drop-shadow-2xl flex flex-col m-auto bg-sky-800">
          <div
            class="absolute top-0 right-1 cursor-pointer px-1.5 py-0.5 hover:bg-sky-700"
            onClick={() => props.close()}
          >
            <FaSolidXmark class="inline-block"></FaSolidXmark>
          </div>
          <div class="relative overflow-y-auto">
            {props.children}
          </div>
        </div>
      </div>
    </Show>
  );
};

export default Modal;
