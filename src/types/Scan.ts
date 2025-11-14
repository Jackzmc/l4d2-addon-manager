export const enum ScanSpeed {
    Maximum = "maximum",
    Normal = "normal",
    Background = "background"
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

export interface ScanProgress {
    items: number,
    processed: number
}

export const enum ScanState {
    Inactive,
    Running,
    Cancelling
}