<template>
<div>
    <div class="logs-container">
        <table class="table is-fullwidth is-narrow is-hoverable">
            <tbody class="is-family-monospace">
                <LogEntryComponent v-for="(log, i) of logs" :key="i" :entry="log" />
            </tbody>
        </table>
    </div>
</div>
</template>

<script setup lang="ts">
import { UnlistenFn } from '@tauri-apps/api/event';
import { attachLogger, LogLevel } from '@tauri-apps/plugin-log';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { getLogs } from '../js/tauri.ts';
import { LogEntry } from '../types/App.ts';
import LogEntryComponent from '../components/LogEntryComponent.vue';

const MAX_LINES = 400

const logs = ref<(LogEntry|ParsedLogEntry)[]>([])
let unsubLogger: UnlistenFn|undefined

function onLogEntry(entry: LogEntry) {
    logs.value.push(parseLogEntry(entry))
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
            module: match[3],
            levelStr: match[4],
            level: entry.level,
            message: match[5]
        }
    } else {
        return entry
    }
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