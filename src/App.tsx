import "./App.css";
import { Route, Router } from "@solidjs/router";

import { Toaster } from "solid-toast";
import Footer from "./components/Footer";
import Home from "./components/Home";
import ProfileEditor from "./components/ProfileEditor";
import ProfileLaunch from "./components/ProfileLaunch";
import Titlebar from "./components/TitleBar";

function App() {
  return (
    <main class="h-full w-full relative flex flex-col overflow-none">
      <Titlebar></Titlebar>
      <div class="flex flex-col flex-grow relative min-h-0 overflow-y-auto" style={{ "scrollbar-gutter": "stable" }}>
        <div class="w-full h-full p-3 flex-grow">
          <Router>
            <Route path="/" component={Home}></Route>
            <Route path="/profile/new" component={ProfileEditor}></Route>
            <Route path="/profile/:id/edit" component={ProfileEditor}></Route>
            <Route path="/profile/:id/play" component={ProfileLaunch}></Route>
          </Router>
        </div>
        <Footer></Footer>
        <Toaster
          position="bottom-center"
          containerClassName="w-full"
          toastOptions={{
            duration: 7500,
            style: { "max-width": "inherit" },
          }}
        />
      </div>
    </main>
  );
}

export default App;
