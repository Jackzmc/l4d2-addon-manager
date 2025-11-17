<template>
<div style="height: 100%">
    <div class="columns is-gapless" style="height: 100%">
        <div class="column is-one-fifth" >
            <Sidebar :scan-state="scanState" :app-data="staticData" :counts="counts" :availableUpdate="updateData"
                @update="update"
                @scan="onScanRequest"
            />
        </div>
        <main class="column mt-0 section-component" >
            <router-view v-slot="{ Component }">
                <Transition>
                    <keep-alive include="Logs">
                            <component ref="view" :is="Component" 
                                :config="config" :static-data="staticData" :availableUpdate="updateData"
                                @check-update="checkForUpdates"
                                @update="update"
                            />
                    </keep-alive>
                </Transition>
            </router-view>
        </main>
    </div>
    <progress v-if="scanState != ScanState.Inactive" 
        :class="['mt-6 progress scan is-small mb-0',
            {'is-info': scanState === ScanState.Running, 'is-warning': scanState === ScanState.Cancelling}]" 
        style="border-radius: 0;" :value="scanProgress?.value" :max="scanProgress?.total"
    />
</div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { notify } from '@kyvg/vue3-notification';
import { computed, onMounted, onUnmounted, ref, Transition } from 'vue';
import { ScanSpeed, ScanState, ScanStateEvent } from '../types/Scan.ts';
import { AddonCounts, AppConfig, ProgressPayload, StaticAppData, UpdateData } from '../types/App.ts'
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { abortScan, countAddons, startScan } from '../js/tauri.ts';
import { check, Update } from '@tauri-apps/plugin-updater';

const availableUpdate = ref<Update|null>(null)
const updatingOrChecking = ref(false)

// Is slow, 1s / addon, so run it infrequently
const BACKGROUND_SCAN_INTERVAL = 1000 * 60 * 60 * 1 // every day?

const props = defineProps<{
    staticData: StaticAppData,
    config: AppConfig
}>()

const updateData = computed(() => {
    return {
        version: availableUpdate.value?.version,
        updating: updatingOrChecking.value
    } as UpdateData
})

const view = ref()
const scanState = ref<ScanState>(ScanState.Inactive)
const scanProgress = ref<ProgressPayload|null>(null)
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

async function checkForUpdates() {
    try {
        updatingOrChecking.value = true
        const update = await check();
        if(!update) return notify({
            type: "info",
            title: "No update found"
        })
        console.info(
            `found update ${update.version} from ${update.date} with notes ${update.body}`
        );
        notify({
            type: "info",
            title: "Update Found",
            text: `v${update.version} is now available`
        })
        availableUpdate.value = update
    } catch(err) {
        console.error(`[Updater] check failed:`, err)
    } finally {
        updatingOrChecking.value = false
    }
}

async function update() {
    if(!availableUpdate.value) throw new Error("No update to update to")
    updatingOrChecking.value = true
    const update = availableUpdate.value
    let downloaded = 0;
    let contentLength = 0;
    // alternatively we could also call update.download() and update.install() separately
    await update.downloadAndInstall((event) => {
        switch (event.event) {
            case 'Started':
                contentLength = event.data.contentLength ?? 0;
                console.log(`started downloading ${event.data.contentLength} bytes`);
                break;
            case 'Progress':
                downloaded += event.data.chunkLength;
                console.log(`downloaded ${downloaded} from ${contentLength}`);
                break;
            case 'Finished':
                console.log('download finished');
                break;
        }
    });
    

    console.log('update installed');
    notify({
        type: "info",
        title: "Update Complete",
        text: "Restart app to launch updated version"
    })
    availableUpdate.value = null
    updatingOrChecking.value = false
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

    stopScanProgressListener = await listen<ProgressPayload>("scan_progress", (event) => {
        // Don't set any progress if we cancelling, want to show the intermediate bar
        if(scanState.value != ScanState.Cancelling)
            scanProgress.value = event.payload
        console.debug("scan_progress", event.payload)
    })

    // Start initial scan
    if(props.staticData.is_prod) startScan(ScanSpeed.Background)
    // Setup background scan, only runs on one thread
    setInterval(() => startScan(ScanSpeed.Background), BACKGROUND_SCAN_INTERVAL)

    checkForUpdates()
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