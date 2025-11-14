import { invoke, InvokeArgs, InvokeOptions } from '@tauri-apps/api/core'
import { AddonEntry } from '../types/Addon.ts';
import { notify } from '@kyvg/vue3-notification';
import { AddonCounts, AppConfig, InitAppData, ItemResult, LogEntry } from '../types/App.ts';
import { handleItemResults } from './app.ts';
import { ScanSpeed } from '../types/Scan.ts';

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

export async function countAddons(): Promise<AddonCounts> {
    const data: number[] = await tryInvoke("addons_counts")
    return {
        addons: data[0],
        workshop: data[1]
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


export async function startScan(speed: ScanSpeed = ScanSpeed.Normal): Promise<void> {
    return await tryInvoke("addons_start_scan", { speed })
}

export async function abortScan(reason?: string): Promise<void> {
    return await tryInvoke("addons_abort_scan", { reason })
}

export async function migrateWorkshopAddons(ids: number[]): Promise<ItemResult[]> {
    const results: ItemResult[] = await tryInvoke("addons_migrate", { ids })
    const errors = handleItemResults(results)
    if(errors === 0) {
        notify({
            type: "success",
            title: "Migration successful",
            text: `${results.length} addons have been moved to trash`
        })
    } else {
        notify({
            type: errors === results.length ? "error" : "warn",
            title: "Migration had errors",
            text: `${errors} / ${results.length} addons failed to be migrated. See logs for info`
        })
    }
    return results
}

export async function unsubscribeAddons(ids: number[]): Promise<ItemResult[]> {
    const results: ItemResult[] = await tryInvoke("addons_unsubscribe", { ids })
    const errors = handleItemResults(results)
    if(errors === 0) {
        notify({
            type: "success",
            title: "Unsubscribe successful",
            text: `${results.length} addons have been unsubscribed`
        })
    } else {
        notify({
            type: errors === results.length ? "error" : "warn",
            title: "Addons had errors",
            text: `${errors} / ${results.length} addons failed to be unsubscribed from. See logs for info`
        })
    }
    return results
}

export async function setAddonState(filenames: string[], state: boolean): Promise<ItemResult[]> {
    const results: ItemResult[] = await tryInvoke("addons_set_state", { filenames, state })
    const errors = handleItemResults(results)
    const stateText = state ? "enabled" : "disabled"
    if(errors === 0) {
        notify({
            type: "success",
            title: `Addons ${stateText} successfully`,
            text: `${results.length} addons have been ${stateText}`
        })
    } else {
        notify({
            type: errors === results.length ? "error" : "warn",
            title: "Addons had errors",
            text: `${errors} / ${results.length} addons failed to be ${stateText}. See logs for info`
        })
    }
    return results
}

export async function deleteAddons(filenames: string[]): Promise<ItemResult[]> {
    const results: ItemResult[] = await tryInvoke("addons_delete", { filenames })
    const errors = handleItemResults(results)
    if(errors === 0) {
        notify({
            type: "success",
            title: "Deletion successful",
            text: `${results.length} addons have been moved to trash`
        })
    } else {
        notify({
            type: errors === results.length ? "error" : "warn",
            title: "Deletion had errors",
            text: `${errors} / ${results.length} addons failed to be deleted. See logs for info`
        })
    }
    return results
}
export async function exportApp(withAddons: boolean): Promise<void> {
    return await tryInvoke("export", { withAddons })
}

export async function resetDatabase(): Promise<void> {
    return await tryInvoke("reset_db")
}

export async function getLogs(): Promise<LogEntry[]> {
    return await tryInvoke("get_logs")
}

export async function openLogsFolder(): Promise<void> {
    return await tryInvoke("open_logs_folder")
}

/** uploads and opens url in browser */
export async function uploadLogs(): Promise<string> {
    return await tryInvoke("upload_logs")
}

export async function addTag(entryId: string, tag: string) {
    return await tryInvoke("addons_tag_add", { id: entryId, tag })
}

export async function removeTag(entryId: string, tag: string) {
    return await tryInvoke("addons_tag_del", { id: entryId, tag })
}