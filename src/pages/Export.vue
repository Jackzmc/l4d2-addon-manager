<template>
    <div class="container px-6">
        <br>
        <p class="subtitle">This will export the app's database and config, and optionally your game addons</p>
        <Field class="box">
            <label class="checkbox">
                <input type="checkbox" class="checkbox" v-model="includeAddons">
                Include Addons Folder
            </label>
            <p class="help">Should the export zip also include a backup of your addons folder? This may take a long time.</p>
            <p class="help is-warning">
                FEATURE IN DEVELOPMENT. IT WILL BE PAINFULLY SLOW
            </p>
        </Field>

        <br>
        <Field>
            <button class="button is-link" @click="startExport" :disabled="isActive ? true : undefined">Start Export</button>
        </Field>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { exportApp } from '../js/tauri.ts';
import { notify } from '@kyvg/vue3-notification';

const includeAddons = ref(false)
const isActive = ref(false)

async function startExport() {
    isActive.value = true
    const path = await exportApp(includeAddons.value)
    isActive.value = false
    notify({
        type: "success",
        title: "Export Complete",
        text: `Exported to ${path}`
    })
}
</script>
