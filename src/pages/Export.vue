<template>
    <div class="container px-6">
        <br>
        <h4 class="title is-4">Export App</h4>
        <p class="subtitle">
            This will export the app's database and config, and optionally your game addons
            <br>
            In future versions, you will be able to import, and restore addons
        </p>
        
        <Field class="box has-background-info-light">
            <label class="checkbox large">
                <input type="checkbox" class="checkbox large" v-model="includeAddons">
                Include Addons Folder
            </label>
            <p>Should the export zip also include a backup of your addons folder? This may take a long time.</p>
        </Field>
        <br>
        <progress v-if="isActive" class="mt-6 progress is-link"  :value="progress?.value" :max="progress?.total">

        </progress>
        <Field v-else>
            <button class="button is-link" @click="startExport" :disabled="isActive ? true : undefined">Start Export</button>
        </Field>

    </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { exportApp } from '../js/tauri.ts';
import { notify } from '@kyvg/vue3-notification';
import Field from '../components/Field.vue';
import { listen } from '@tauri-apps/api/event';
import { ProgressPayload } from '../types/App.ts';

const progress = ref<ProgressPayload|null>(null)
const includeAddons = ref(false)
const isActive = ref(false)

async function startExport() {
    progress.value = null
    isActive.value = true
    const stopListener = await listen<ProgressPayload>("export_progress", event => {
        progress.value = event.payload
    })
    try {
        const path = await exportApp(includeAddons.value)
        notify({
            type: "success",
            title: "Export Complete",
            text: `Exported to ${path}`
        })
        isActive.value = false
    } catch(err) {
        // just to ensure stopListener can be called
    }
    stopListener()
}
</script>
