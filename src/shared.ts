import { writable } from "svelte/store";

export type Config = {
  deepSeekKey: string;
  aliKey: string;
};

export let config = writable({} as Config);
