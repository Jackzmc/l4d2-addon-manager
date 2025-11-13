<template>
<div class="mt-2">
    <div class="container px-4 buttons">
        <button :class="['button','is-link','is-outlined']" @click="openLogsFolder()">Open Logs Folder</button>
        <button :class="['button','is-link','is-outlined',{'is-loading': isLogsUploading}]" @click="onUploadLogsPressed">Upload Logs</button>
    </div>
    <div class="logs-container">
        <table class="table is-fullwidth is-narrow is-hoverable">
            <thead>
                <tr>
                    <th>Time</th>
                    <th>Module</th>
                    <th>Level</th>
                    <th>Message</th>
                </tr>
            </thead>
            <tbody class="is-family-monospace">
                <LogEntryComponent v-for="(log, i) of logs" :key="i" :entry="log" />
            </tbody>
        </table>
    </div>
    <br>
</div>
</template>

<script setup lang="ts">
import { UnlistenFn } from '@tauri-apps/api/event';
import { attachLogger } from '@tauri-apps/plugin-log';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { getLogs, openLogsFolder, uploadLogs } from '../js/tauri.ts';
import { LogEntry } from '../types/App.ts';
import LogEntryComponent from '../components/LogEntryComponent.vue';

const MAX_LINES = 400

const logs = ref<(LogEntry|ParsedLogEntry)[]>([])
const isLogsUploading = ref(false)
let unsubLogger: UnlistenFn|undefined

function onLogEntry(entry: LogEntry) {
    logs.value.unshift(parseLogEntry(entry))
    if(logs.value.length > MAX_LINES) {
        logs.value.splice(MAX_LINES)
    }
}

const LOG_ENTRY_REGEX = new RegExp(/^\[(\d{4}-\d{2}-\d{2})\]\[(\d{2}:\d{2}:\d{2})\]\[([a-zA-Z0-9_:]+)\]\[([A-Z]+)] (.*)/)
export interface ParsedLogEntry extends LogEntry {
    /** yyyy-mm-dd */
    date?: string 
    /** 24hr */
    time?: string,
    module?: string,
    levelStr?: string
}

function parseLogEntry(entry: LogEntry): ParsedLogEntry | LogEntry {
    const match = entry.message.match(LOG_ENTRY_REGEX)
    if(match) {
        return {
            date: match[1],
            time: match[2],
            module: match[3].replace("l4d2_addon_manager_lib", "app"),
            levelStr: match[4],
            level: entry.level,
            message: match[5]
        }
    } else {
        return entry
    }
}

async function onUploadLogsPressed() {
    if(isLogsUploading.value) return
    isLogsUploading.value = true
    await uploadLogs()
    isLogsUploading.value = false
}

onMounted(async () => {
    unsubLogger = await attachLogger(onLogEntry)
    const entries = (await getLogs())
    entries.forEach(entry => onLogEntry(entry))
})

onBeforeUnmount(() => {
    if(unsubLogger) unsubLogger()
})
</script>