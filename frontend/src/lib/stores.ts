import { writable } from "svelte/store";
import type { User } from "$lib/types";

export const user = writable<User | null>(null);
