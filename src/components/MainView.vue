<template>
    <div class="columns is-gapless">
        <div class="column is-3" >
            <Sidebar @scan="onScanRequest" :scan-active="isScanActive" :app-data="staticData" :counts="counts" />
        </div>
        <main class="column mt-3 section-component">
            <router-view v-slot="{ Component }">
                <Transition>
                    <keep-alive include="Logs">
                            <component ref="view" :is="Component" :config="config" />
                    </keep-alive>
                </Transition>
            </router-view>
        </main>
    </div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { notify } from '@kyvg/vue3-notification';
import { onMounted, ref, Transition } from 'vue';
import { AddonCounts, AppConfig, ScanResultEvent, ScanResultMessage, ScanSpeed, ScanStateEvent, StaticAppData } from '../types/App.ts';
import { listen } from '@tauri-apps/api/event';
import { abortScan, countAddons, startScan } from '../js/tauri.ts';

// Is slow, 1s / addon, so run it infrequently
const BACKGROUND_SCAN_INTERVAL = 1000 * 60 * 60 * 1 // every day?

const props = defineProps<{
    staticData: StaticAppData,
    config: AppConfig
}>()

const view = ref()
const isScanActive = ref(false)
const counts = ref<AddonCounts>({ addons: 0, workshop: 0 })

// tell child to refresh, if they can
async function triggerPageRefresh() {
    if(view.value?.refresh) {
        view.value.refresh()
        counts.value = await countAddons()
    }
}

async function onScanRequest() {
    if(isScanActive.value) {
        await abortScan("requested by user")
    } else {
        await startScan()
    }
}

onMounted(async() => {
    counts.value = await countAddons()
    await listen<ScanStateEvent>("scan_state", (event) => {
        console.debug("scan_state", event)
        if(event.payload.state === "started") {
            notify({
                type: "info",
                title: `Scan started`,
                text: "Scan has started in the background"
            })
            isScanActive.value = true
        } else if(event.payload.state === "aborted") {
            notify({
                type: "warn",
                title: `Scan cancelled`,
                text: `Reason: ${event.payload.reason ?? "(None)"}`
            })
            isScanActive.value = false
        } else if(event.payload.state === "complete") {
            const type = event.payload.failed > 0 ? "warn" : "success"
            notify({
                type: type,
                title: `Scan complete ${(event.payload.failed > 0 ) ? 'with errors' : ''}`,
                text: `${event.payload.total} files scanned, ${event.payload.added} new addons found, ${event.payload.failed} errors\nSee logs for details`
            })
            triggerPageRefresh()
            isScanActive.value = false
        }
    })

    await listen<ScanResultEvent>("scan_result", (event) => {
        const data = ScanResultMessage[event.payload.result]
        if(data) {
            notify({
                type: "info",
                title: data.title,
                text: event.payload.filename
            })
        }
    })

    // Start initial scan
    startScan()
    // Setup background scan, only runs on one thread
    setInterval(() => startScan(ScanSpeed.Background), BACKGROUND_SCAN_INTERVAL)
})
</script>

<style>
.section-component {
  height: 720px !important;
  overflow: auto !important;
}
</style>