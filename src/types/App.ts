import { LogLevel } from "@tauri-apps/plugin-log"

export interface SetRoute {
    name?: string
}

export interface AddonCounts {
    addons: number,
    workshop: number
}

export interface InitAppData {
    initial_route: SetRoute,
    data: StaticAppData,
    config: AppConfig
}
export interface StaticAppData {
    app_version: string,
    git_commit: string | null
}

export interface AppConfig {
    addons_folder: string | null,
    steam_apikey: string | null
}

export type ScanStateEvent = ScanStateEvent_Started | ScanStateEvent_Aborted | ScanStateEvent_Complete
export interface ScanStateEvent_Started {
    state: "started",
    speed: ScanSpeed
}
export interface ScanStateEvent_Aborted {
    state: "aborted",
    reason?: string
}
export interface ScanStateEvent_Complete {
    state: "complete",
    time: number // seconds
    total: number,
    added: number,
    updated: number,
    failed: number
}

export type ScanResultType = "updated" | "renamed" | "added" | "no_action"
export interface ScanResultEvent {
    result: ScanResultType,
    filename: string
}
export const ScanResultMessage: Record<ScanResultType, { title: string } | undefined> = {
    // Don't show these:
    updated: undefined,
    no_action: undefined,

    ["added"]: {
        title: "New Addon Found"
    },
    ["renamed"]: {
        title: "Found Renamed Addon"
    }
}

export type ItemResult = ItemResult_Ok | ItemResult_Error
export interface ItemResult_Ok {
    result: "ok",
    filename: string
}
export interface ItemResult_Error {
    result: "error",
    filename: string,
    error: string
}

export interface LogEntry {
    message: string, 
    level: LogLevel
}

export const enum ScanSpeed {
    Maximum = "maximum",
    Normal = "normal",
    Background = "background"
} 