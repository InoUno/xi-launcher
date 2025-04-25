import { useNavigate } from "@solidjs/router";
import { FaSolidCopy, FaSolidPencil, FaSolidPlay } from "solid-icons/fa";
import { For } from "solid-js";
import { commands } from "../bindings";
import { useData } from "../store";

const ProfileList = () => {
  const { profiles, profilesRefetch } = useData();
  const navigate = useNavigate();

  const launchProfile = (id: number) => {
    navigate(`/profile/${id}/play`);
  };

  const duplicateProfile = async (id: number) => {
    await commands.duplicateProfile(id);
    await profilesRefetch();
  };

  return (
    <div>
      <h2>Profiles</h2>
      <ul class="border-1 border-sky-500 divide-y divide-sky-500 divide-solid">
        <For each={profiles.ids}>
          {id => {
            const item = profiles.map?.[id];
            if (!item) {
              return <li>Error</li>;
            }

            return (
              <li class="cursor-pointer flex flex-row h-10
                  not-hover:bg-gradient-to-r from-sky-800 to-sky-900 hover:bg-sky-700">
                <div
                  class="cursor-pointer flex flex-grow group"
                  draggable={true}
                  onDblClick={() => launchProfile(id)}
                >
                  <div
                    class="text-xl text-center w-10 h-full group-hover:text-green-300"
                    onClick={() => launchProfile(id)}
                  >
                    <FaSolidPlay class="inline-block mt-2"></FaSolidPlay>
                  </div>
                  <div class="py-2">
                    {item.name ?? <span class="italic">Missing name</span>}
                  </div>
                </div>
                <div
                  class="cursor-pointer text-xl text-center w-12 h-full hover:text-amber-300"
                  onClick={() => navigate(`/profile/${id}/edit`)}
                >
                  <FaSolidPencil class="inline-block mt-2"></FaSolidPencil>
                </div>
                <div
                  class="cursor-pointer text-xl text-center w-12 h-full hover:text-blue-300"
                  onClick={() => duplicateProfile(id)}
                >
                  <FaSolidCopy class="inline-block mt-2"></FaSolidCopy>
                </div>
              </li>
            );
          }}
        </For>
      </ul>

      <div class="mt-2">
        <button
          class="button accept w-full"
          onClick={() => navigate(`/profile/new`)}
        >
          New
        </button>
      </div>
    </div>
  );
};

export default ProfileList;
