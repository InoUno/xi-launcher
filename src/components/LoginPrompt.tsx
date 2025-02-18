import { createSignal, onMount } from "solid-js";

interface LoginPromptProps {
  callback: (password: string) => any;
}

export interface LoginQuery {
  index: number;
}

const LoginPrompt = (props: LoginPromptProps) => {
  const [getPassword, setPassword] = createSignal<string>("");

  let passwordRef: HTMLInputElement;
  onMount(() => {
    passwordRef.focus();
  });

  return (
    <div class="w-full m-auto form">
      <form onSubmit={e => e.preventDefault()}>
        <div class="field">
          <label class="label" for="password">
            Password
          </label>
          <input
            id="password"
            type="password"
            placeholder="Password"
            ref={passwordRef!}
            value={getPassword()}
            onInput={e => setPassword(e.target.value ?? "")}
          >
          </input>
        </div>
        <button
          type="submit"
          class="button accept w-full mt-2"
          onClick={() => {
            props.callback(getPassword());
          }}
        >
          Login
        </button>
      </form>
    </div>
  );
};

export default LoginPrompt;
