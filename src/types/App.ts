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

export interface ProgressPayload {
    value: number,
    total: number
}