import { useNavigate, useParams } from "@solidjs/router";
import { createEffect, createResource, createSignal, Match, Switch } from "solid-js";
import { unwrap } from "solid-js/store";
import toast from "solid-toast";
import { commands } from "../bindings";
import { useData } from "../store";
import { unwrapResult } from "../util";
import Installer from "./Installer";
import LoginPrompt from "./LoginPrompt";
import Updater from "./Updater";

const ProfileLaunch = () => {
  const params = useParams();
  const id = parseInt(params.id);

  const navigate = useNavigate();

  const { profiles } = useData();

  const profile = profiles.map?.[id];
  if (!profile) {
    navigate("/");
    return;
  }

  const [getAutoLaunch, setAutoLaunch] = createSignal<boolean>(true);

  const [resource, { refetch: refetchStatus }] = createResource(async () => await commands.checkLaunchProfile(id));

  createEffect(() => {
    if (resource.loading) {
      return;
    }
    const result = resource();
    if (result?.status == "error") {
      toast.error(result.error as string);
      navigate("/");
      return;
    }
  });

  const [isLaunching, setIsLaunching] = createSignal<boolean>(false);
  const launch = async (id: number, password: string | null = null) => {
    if (!isLaunching()) {
      setIsLaunching(true);
      try {
        await commands.launchProfile(id, password).then(result => {
          unwrapResult(result);
          navigate("/");
        });
      } catch (err) {
        console.error(err);
        toast.error(err as string);
      } finally {
        setIsLaunching(false);
      }
    }
  };

  const actionComponent = () => {
    const result = resource();
    if (!result) {
      return;
    }

    if (result?.status == "error") {
      toast.error(result.error as string);
      navigate("/");
      return;
    }

    switch (result.data.type) {
      case "NeedsGameDir":
        toast.error("Missing game directory.");
        navigate(`/profile/${id}/edit`);
        break;

      case "NeedsInstall":
        toast.error(
          "Server does not provide an install URL. Please install the game yourself in the folder you specified.",
        );
        navigate(`/`);
        break;

      case "NeedsAndCanInstall":
        setAutoLaunch(false);
        return (
          <Installer
            profile={unwrap(profile)}
            downloadInfo={result.data.data.download_info}
            isComplete={() => {
              refetchStatus();
            }}
          />
        );

      case "NeedsAshita":
        toast.error("Missing Ashita directory.");
        navigate(`/profile/${id}/edit`);
        break;

      case "NeedsWindower":
        toast.error("Missing Windower directory.");
        navigate(`/profile/${id}/edit`);
        break;

      case "NeedsUpdate":
        setAutoLaunch(false);
        return (
          <Updater
            id={id}
            isComplete={() => {
              refetchStatus();
            }}
          />
        );

      case "NeedsPassword":
        return (
          <LoginPrompt
            callback={password => {
              launch(id, password);
            }}
          >
          </LoginPrompt>
        );

      case "Ready":
      default:
        if (getAutoLaunch()) {
          launch(id);
        } else {
          return (
            <form
              onSubmit={e => {
                e.preventDefault();
                launch(id);
              }}
            >
              <button class="button accept" type="submit">
                Launch
              </button>
            </form>
          );
        }
    }
  };

  return (
    <div class="w-full h-full flex flex-col items-center">
      <div class="w-full flex-grow">
        <Switch>
          <Match when={resource.loading}>
            Checking game launch...
          </Match>
          <Match when={!resource() || resource()?.status == "error"}>
            Error while checking for game launch.
          </Match>
          <Match when={resource() && resource()?.status == "ok"}>
            {actionComponent()}
          </Match>
        </Switch>
      </div>
      <button
        class="button w-full my-2"
        onClick={() => {
          navigate(`/`);
        }}
      >
        Back
      </button>
    </div>
  );
};

export default ProfileLaunch;
