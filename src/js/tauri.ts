import { invoke, InvokeArgs, InvokeOptions } from '@tauri-apps/api/core'
import { Addon } from '../types/Addon.ts';
import { notify } from '@kyvg/vue3-notification';
import { InitAppData } from '../types/App.ts';

async function tryInvoke<T>(cmd: string, args?: InvokeArgs, options?: InvokeOptions): Promise<T> {
    try {
        return await invoke(cmd, args, options)
    } catch(err: any) {
        console.error("[TAURI] [ERROR]", err)
        notify({
            type: "error",
            title: "Backend Error",
            text: `An error occurred while running ${cmd}: ${err.message ?? err}`,
        });
        throw err
    }
}

export async function getManagedAddons(): Promise<Addon[]> {
    return await tryInvoke("get_managed_addons")
}

export async function getGameFolder(): Promise<string | null> {
    return await tryInvoke("choose_game_folder")
}

export async function setGameFolder(path: string): Promise<void> {
    return await tryInvoke("set_game_folder", { path })
}

export async function init(): Promise<InitAppData> {
    return await tryInvoke("init")
}