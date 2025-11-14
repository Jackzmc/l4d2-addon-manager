<template>
<div>
    <div class="columns is-gapless" style="height: 100%">
        <div class="column is-one-fifth" >
            <Sidebar @scan="onScanRequest" :scan-state="scanState" :app-data="staticData" :counts="counts" />
        </div>
        <main class="column mt-0 section-component" >
            <router-view v-slot="{ Component }">
                <Transition>
                    <keep-alive include="Logs">
                            <component ref="view" :is="Component" :config="config" :static-data="staticData" />
                    </keep-alive>
                </Transition>
            </router-view>
        </main>
    </div>
    <progress v-if="scanState != ScanState.Inactive" :class="['mt-6 progress scan is-small mb-0',{'is-info': scanState === ScanState.Running, 'is-warning': scanState === ScanState.Cancelling}]" style="border-radius: 0;" :value="scanProgress?.processed" :max="scanProgress?.items">

    </progress>
</div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { notify } from '@kyvg/vue3-notification';
import { onMounted, onUnmounted, ref, Transition } from 'vue';
import { ScanSpeed, ScanState, ScanStateEvent } from '../types/Scan.ts';
import { AddonCounts, AppConfig, StaticAppData } from '../types/App.ts'
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { abortScan, countAddons, startScan } from '../js/tauri.ts';
import { ScanProgress } from '../types/Scan.ts';

// Is slow, 1s / addon, so run it infrequently
const BACKGROUND_SCAN_INTERVAL = 1000 * 60 * 60 * 1 // every day?

const props = defineProps<{
    staticData: StaticAppData,
    config: AppConfig
}>()

const view = ref()
const scanState = ref<ScanState>(ScanState.Inactive)
const scanProgress = ref<ScanProgress|null>(null)
const counts = ref<AddonCounts>({ addons: 0, workshop: 0 })

// tell child to refresh, if they can
async function triggerPageRefresh() {
    if(view.value?.refresh) {
        view.value.refresh()
        counts.value = await countAddons()
    }
}

async function onScanRequest() {
    switch(scanState.value) {
        case ScanState.Inactive:
            await startScan()
            break
        case ScanState.Running:
            scanState.value = ScanState.Cancelling
            scanProgress.value = null
            await abortScan("requested by user")
            scanState.value = ScanState.Inactive
            break
    }
}


let stopScanStateListener: UnlistenFn|undefined
let stopScanProgressListener: UnlistenFn|undefined
onMounted(async() => {
    counts.value = await countAddons()
    stopScanStateListener = await listen<ScanStateEvent>("scan_state", (event) => {
        console.debug("scan_state", event)
        if(event.payload.state === "started") {
            notify({
                type: "info",
                title: `Scan started`,
                text: "Scan has started in the background"
            })
            scanState.value = ScanState.Running
        } else if(event.payload.state === "aborted") {
            notify({
                type: "warn",
                title: `Scan cancelled`,
                text: `Reason: ${event.payload.reason ?? "(None)"}`
            })
            scanState.value = ScanState.Inactive
        } else if(event.payload.state === "complete") {
            const type = event.payload.failed > 0 ? "warn" : "success"
            notify({
                type: type,
                title: `Scan completed in ${event.payload.time} seconds ${(event.payload.failed > 0 ) ? 'with errors' : ''}`,
                text: `${event.payload.total} files scanned, ${event.payload.added} new addons found, ${event.payload.failed} errors\nSee logs for details`
            })
            triggerPageRefresh()
            scanState.value = ScanState.Inactive
        }
        scanProgress.value = null
    })

    stopScanProgressListener = await listen<ScanProgress>("scan_progress", (event) => {
        // Don't set any progress if we cancelling, want to show the intermediate bar
        if(scanState.value != ScanState.Cancelling)
            scanProgress.value = event.payload
        console.debug("scan_progress", event.payload)
    })

    // Start initial scan
    startScan(ScanSpeed.Background)
    // Setup background scan, only runs on one thread
    setInterval(() => startScan(ScanSpeed.Background), BACKGROUND_SCAN_INTERVAL)
})

onUnmounted(() => {
    if(stopScanProgressListener) stopScanProgressListener()
    if(stopScanStateListener) stopScanStateListener()
})
</script>

<style>
.section-component {
  /* height: 720px !important; */
  overflow: auto !important;
}
.progress.scan {
  position: fixed;
  bottom: 0;
}
</style>