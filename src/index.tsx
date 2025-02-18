/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import { DataProvider } from "./store";

render(
  () => (
    <DataProvider>
      <App />
    </DataProvider>
  ),
  document.getElementById("root") as HTMLElement,
);
