<template>
<div class="container has-text-centered mt-6">
    <div class="block">
        <h1 class="title is-1">L4D2 Addon Manager</h1>
        <p class="subtitle is-2">Version {{ staticData.app_version }} <template v-if="bundleType">({{ bundleType }})</template></p>
        <div class="block">
            <p class="subtitle is-4 mb-0">Credits</p>
            <ul>
                <li><b><a href="https://tauri.app">Tauri</a></b> v{{ tauriVersion }}</li>
                <li>Icons <b><a href="https://iconoir.com/">Iconoir</a></b></li>
            </ul>
        </div>
        <div class="block my-4">
            <nav class="level">
                <p class="level-item has-text-centered">
                    <Button href="https://git.jackz.me/jackz/l4d2-addon-manager" class="link" icon-left="git" icon-right="open-new-window">Source Code</Button>
                </p>
                <p class="level-item has-text-centered">
                    <Button v-if="availableUpdate.version" color="is-link" :loading="availableUpdate.updating" icon-left="iconoir:download" @click="update">Install Update v{{ availableUpdate.version }}</Button>
                    <Button v-else color="is-link" :loading="availableUpdate.updating" icon-left="iconoir:refresh" @click="checkForUpdates">Check for updates</Button>

                </p>
            </nav>
        </div>
    </div>
</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { StaticAppData, UpdateData } from '../types/App.ts';
import Button from '../components/Button.vue';
import { notify } from '@kyvg/vue3-notification';
import { BundleType, getBundleType, getTauriVersion } from '@tauri-apps/api/app';

const emit = defineEmits(["check-update", "update"])

const props = defineProps<{
    staticData: StaticAppData,
    availableUpdate: UpdateData
}>()

const tauriVersion = ref<string>()
const bundleType = ref<BundleType>()

function checkForUpdates() {
    emit("check-update")
}

function update() { 
    emit("update")
}

onMounted(async() => {
    bundleType.value = (await getBundleType())
    tauriVersion.value = await getTauriVersion()
})
</script>