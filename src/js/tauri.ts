import { invoke, InvokeArgs, InvokeOptions } from '@tauri-apps/api/core'
import { AddonEntry } from '../types/Addon.ts';
import { notify } from '@kyvg/vue3-notification';
import { AppConfig, InitAppData } from '../types/App.ts';

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

export async function listAddons(workshop = false): Promise<AddonEntry[]> {
    return await tryInvoke(workshop ? "addons_list_workshop" : "addons_list_managed")
}

export async function getGameFolder(): Promise<string | null> {
    return await tryInvoke("choose_game_folder")
}

export async function setGameFolder(path: string): Promise<void> {
    return await tryInvoke("set_game_folder", { path })
}
/** Replaces app config if successfully validates. Throws err otherwise */
export async function setConfig(config: AppConfig): Promise<void> {
    return await tryInvoke("set_config", { config })
}

export async function init(): Promise<InitAppData> {
    return await tryInvoke("init")
}


export async function startScan(): Promise<void> {
    return await tryInvoke("addons_start_scan")
}

export async function abortScan(reason?: string): Promise<void> {
    return await tryInvoke("addons_abort_scan", { reason })
}

export async function migrateWorkshopAddons(ids: number[]): Promise<void> {
    return await tryInvoke("addons_migrate", { ids })
}

export async function unsubscribeAddons(ids: number[]): Promise<void> {
    return await tryInvoke("addons_unsubscribe", { ids })
}

export async function setAddonState(filenames: string[], state: boolean): Promise<void> {
    return await tryInvoke("addons_set_state", { filenames, state })
}

export async function deleteAddons(filenames: string[]): Promise<void> {
    return await tryInvoke("addons_delete", { filenames })
}
export async function exportApp(withAddons: boolean): Promise<void> {
    return await tryInvoke("export", { withAddons })
}