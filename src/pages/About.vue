<template>
<div class="container has-text-centered mt-6">
    <div class="block">
        <h1 class="title is-1">L4D2 Addon Manager</h1>
        <p class="subtitle is-2">Version {{ staticData.app_version }} <template v-if="bundleType">({{ bundleType }})</template></p>
        <p>
            Tauri v{{ tauriVersion }}
        </p>
        <div class="block my-4">
            <nav class="level">
                <p class="level-item has-text-centered">
                    <Button href="https://git.jackz.me/jackz/l4d2-addon-manager" class="link" icon-left="git" icon-right="open-new-window">Source Code</Button>
                </p>
                <p class="level-item has-text-centered">
                    <Button color="is-link" :loading="isChecking" icon-left="iconoir:refresh" @click="checkForUpdates">Check for updates</Button>
                </p>
            </nav>
        </div>
    </div>
</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { StaticAppData } from '../types/App.ts';
import Button from '../components/Button.vue';
import { notify } from '@kyvg/vue3-notification';
import { BundleType, getBundleType, getTauriVersion } from '@tauri-apps/api/app';

const isChecking = ref(false)

const props = defineProps<{
    staticData: StaticAppData
}>()

const tauriVersion = ref<string>()
const bundleType = ref<BundleType>()

async function checkForUpdates() {
    isChecking.value = true
    notify({
        type: "error",
        title: "Update Check",
        text: "Not implemented"
    })
    isChecking.value = false
}

onMounted(async() => {
    bundleType.value = (await getBundleType())
    tauriVersion.value = await getTauriVersion()
})
</script>