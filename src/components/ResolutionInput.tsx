import { batch, createEffect, createSignal, For, Match, on, onMount, Switch } from "solid-js";

export interface ResolutionInputProps {
  label: string;
  initial?: Resolution;
  onChange: (width: number, height: number) => any;
}

interface Resolution {
  width: number;
  height: number;
}

const enum Mode {
  Preset = 1,
  Manual = 2,
  _16_9 = 3,
  _16_10 = 4,
  _4_3 = 5,
}

const ResolutionInput = (props: ResolutionInputProps) => {
  const { initial } = props;

  const [getWidth, setWidth] = createSignal<number>(initial?.width ?? 1920);
  const [getHeight, setHeight] = createSignal<number>(initial?.height ?? 1080);
  const [getPresetIdx, setPresetIdx] = createSignal<number>(0);

  const presets: (Resolution & { ratio?: string; })[] = [
    { width: 640, height: 480 },
    { width: 960, height: 540 },
    { width: 960, height: 720 },
    { width: 1024, height: 768 },
    { width: 1280, height: 720 },
    { width: 1920, height: 1080 },
    { width: 1920, height: 1440 },
    { width: 2560, height: 1440 },
    { width: 2560, height: 1920 },
    { width: 3840, height: 2160 },
    { width: 3840, height: 2880 },
  ];

  for (const preset of presets) {
    const ratio = preset.width / preset.height;
    if (ratio == 16 / 9) {
      preset.ratio = "16:9";
    } else if (ratio == 16 / 10) {
      preset.ratio = "16:10";
    } else if (ratio == 4 / 3) {
      preset.ratio = "4:3";
    }
  }

  const [getMode, setMode] = createSignal<Mode>(Mode.Preset);

  onMount(() => {
    let foundPreset = false;
    for (const idx in presets) {
      const preset = presets[idx];
      if (preset.width == getWidth() && preset.height == getHeight()) {
        setPresetIdx(idx as any);
        foundPreset = true;
        break;
      }
    }

    let mode = foundPreset ? Mode.Preset : Mode.Manual;
    if (!foundPreset && initial) {
      const ratio = initial.width / initial.height;
      if (ratio == 16 / 9) {
        mode = Mode._16_9;
      } else if (ratio == 16 / 10) {
        mode = Mode._16_10;
      } else if (ratio == 4 / 3) {
        mode = Mode._4_3;
      }
    }

    setMode(mode);
  });

  createEffect(on(getMode, mode => {
    switch (mode) {
      case Mode.Preset:
        let closestIdx = 0;
        let closestDiff = Number.MAX_SAFE_INTEGER;
        for (const idx in presets) {
          const preset = presets[idx];
          let diff = Math.abs(getWidth() - preset.width);
          if (diff < closestDiff) {
            closestIdx = parseInt(idx);
            closestDiff = diff;
          }
        }
        batch(() => {
          setWidth(presets[closestIdx].width);
          setHeight(presets[closestIdx].height);
          setPresetIdx(closestIdx);
        });
        break;
      case Mode._16_9:
        setHeight(Math.round(getWidth() * 9 / 16));
        break;
      case Mode._16_10:
        setHeight(Math.round(getWidth() * 10 / 16));
        break;
      case Mode._4_3:
        setHeight(Math.round(getWidth() * 3 / 4));
        break;
      case Mode.Manual:
        break;
    }
  }, { defer: true }));

  createEffect(on(getHeight, height => {
    switch (getMode()) {
      case Mode._16_9:
        setWidth(Math.round(height * 16 / 9));
        break;
      case Mode._16_10:
        setWidth(Math.round(height * 16 / 10));
        break;
      case Mode._4_3:
        setWidth(Math.round(height * 4 / 3));
        break;
    }
  }));

  createEffect(on(getWidth, width => {
    switch (getMode()) {
      case Mode._16_9:
        setHeight(Math.round(width * 9 / 16));
        break;
      case Mode._16_10:
        setHeight(Math.round(width * 10 / 16));
        break;
      case Mode._4_3:
        setHeight(Math.round(width * 3 / 4));
        break;
    }
  }, { defer: true }));

  createEffect(() => {
    props.onChange(getWidth(), getHeight());
  }, { defer: true });

  return (
    <div class="w-full">
      <label class="label">
        {props.label}
      </label>
      <div class="w-full flex flex-row gap-2">
        <div class="w-full">
          <Switch>
            <Match when={getMode() == Mode.Preset}>
              <select
                onChange={e => {
                  const v = presets[e.target.value as any];
                  batch(() => {
                    setWidth(v.width);
                    setHeight(v.height);
                  });
                }}
                class="text-center"
                value={getPresetIdx()}
              >
                <For each={presets}>
                  {(v, idx) => (
                    <option value={idx()}>
                      {v.width} x {v.height} {v.ratio ? `(${v.ratio})` : ""}
                    </option>
                  )}
                </For>
              </select>
            </Match>

            <Match when={getMode() != Mode.Preset}>
              <div class="w-full flex flex-row gap-1 items-center">
                <div class="w-1/2">
                  <input
                    min={60}
                    type="number"
                    step={10}
                    value={getWidth()}
                    onChange={e => {
                      setWidth(parseInt(e.target.value));
                    }}
                    class="text-center"
                  >
                  </input>
                </div>
                x
                <div class="w-1/2">
                  <input
                    min={60}
                    step={10}
                    type="number"
                    value={getHeight()}
                    onChange={e => {
                      setHeight(parseInt(e.target.value));
                    }}
                    class="text-center"
                  >
                  </input>
                </div>
              </div>
            </Match>
          </Switch>
        </div>
        <div class="w-28">
          <select
            onChange={e => {
              setMode(parseInt(e.target.value) as Mode);
            }}
            value={getMode()}
          >
            <option value={Mode.Preset} class="p-0">Preset</option>
            <option value={Mode.Manual}>Manual</option>
            <option value={Mode._16_9}>16:9</option>
            <option value={Mode._16_10}>16:10</option>
            <option value={Mode._4_3}>4:3</option>
          </select>
        </div>
      </div>
    </div>
  );
};

export default ResolutionInput;
