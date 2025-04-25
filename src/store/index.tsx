import { createContext, FlowProps, useContext } from "solid-js";
import { createProfilesStore } from "./profiles";

function makeDataContext() {
  return {
    ...createProfilesStore(),
  };
}

export type DataContextType = ReturnType<typeof makeDataContext>;
const DataContext = createContext<DataContextType>();

export function DataProvider(props: FlowProps) {
  return (
    <DataContext.Provider value={makeDataContext()}>
      {props.children}
    </DataContext.Provider>
  );
}

export function useData() {
  return useContext(DataContext)!;
}
