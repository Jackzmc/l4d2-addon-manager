<template>
<table class="table is-fullwidth has-sticky-header">
    <thead>
        <tr>
            <td colspan="4">
                <div class="buttons">
                <button class="button" @click="refresh">Refresh</button>
                <button class="button" @click="startScan">Rescan</button>
                </div>
            </td>
        </tr>
        <tr>
            <th>File Name</th>
            <th>File Size</th>
            <th>Info</th>
            <th>Tags</th>
        </tr>
    </thead>
    <tbody>
        <AddonRow v-for="entry in props.addons" :key="entry.addon.filename" :entry="entry" />
    </tbody>
</table>
</template>

<script setup lang="ts">
import { notify } from '@kyvg/vue3-notification';
import { scanAddons } from '../js/tauri.ts';
import { AddonEntry } from '../types/Addon.ts';
import AddonRow from './AddonRow.vue';

const emit = defineEmits(["refresh"])

const props = defineProps<{
    addons: AddonEntry[]
}>()

function refresh() {
    emit("refresh")
}
async function startScan() {
    try {
        await scanAddons(false)
        notify({
            type: "info",
            title: "Scan started",
            text: "Scan has started in the background. This may take some time."
        })
    } catch(err) {
        notify({
            type: "error",
            title: "Scan failed",
            text: err ?? err.message
        })
    }
}
</script>