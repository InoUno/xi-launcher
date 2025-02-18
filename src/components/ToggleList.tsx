import { FaRegularCircle, FaSolidCircle } from "solid-icons/fa";
import { createEffect, createMemo, createResource, createSignal, For, Show, Signal } from "solid-js";
import { createStore, produce, reconcile, unwrap } from "solid-js/store";

export interface Option {
  name: string;
  selected: boolean;
}

export interface ToggleListProps {
  options: Option[];
  onComplete: (selected: string[]) => any;
}

const ToggleList = (props: ToggleListProps) => {
  const [options, setOptions] = createStore(props.options);

  return (
    <form
      onSubmit={e => {
        e.preventDefault();
        props.onComplete(options.map(x => x.name) ?? []);
      }}
      class="w-full h-full flex flex-col"
    >
      <div class="flex-grow min-h-0 overflow-y-auto">
        <ul>
          <For each={options}>
            {(item, idx) => (
              <li
                class="cursor-pointer p-2 rounded-lg hover:bg-sky-700"
                onClick={() => {
                  setOptions(idx(), produce(i => i.selected = !i.selected));
                }}
              >
                <Show
                  when={item.selected}
                  fallback={<FaRegularCircle class="inline-block mr-2 text-yellow-600"></FaRegularCircle>}
                >
                  <FaSolidCircle class="inline-block mr-2 text-green-600"></FaSolidCircle>
                </Show>
                {item.name}
              </li>
            )}
          </For>
        </ul>
      </div>

      <div>
        <button
          type="submit"
          class="button accept w-full"
        >
          Save
        </button>
      </div>
    </form>
  );
};

export default ToggleList;
