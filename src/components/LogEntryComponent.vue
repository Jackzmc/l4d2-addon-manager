<template>
    <tr :class="['is-size-7',entry.levelStr && LEVEL_COLOR[entry.levelStr]]">
        <td>{{ entry.time }}</td>
        <td>{{ entry.module }}</td>
        <td>{{ entry.levelStr}}</td>
        <td>{{ entry.message }}</td>
    </tr>
</template>

<script setup lang="ts">
import { ParsedLogEntry } from '../pages/Logs.vue';

const LEVEL_COLOR: Record<string, string> = {
    ["TRACE"]: 'is-light',
    ["DEBUG"]: '',
    ["INFO"]: 'is-info',
    ["WARN"]: 'is-warning',
    ["ERROR"]: 'is-danger'
}

const props = defineProps<{
    entry: ParsedLogEntry
}>()
</script>