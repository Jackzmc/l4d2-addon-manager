<template>
    <div class="columns is-gapless">
        <div class="column is-3" >
            <Sidebar @refresh-list="onRefreshRequest" />
        </div>
        <main class="column mt-3 section-component">
            <router-view v-slot="{ Component }">
                <component ref="view" :is="Component" />
            </router-view>
        </main>
    </div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { notify } from '@kyvg/vue3-notification';
import { onMounted, ref } from 'vue';
import { ScanResultEvent, ScanResultMessage, ScanStateEvent } from '../types/App.ts';
import { listen } from '@tauri-apps/api/event';

const view = ref()


// If a refresh is requested (from sidebar), tell child to refresh, if they can
function onRefreshRequest() {
    if(view.value.refresh) {
        view.value.refresh()
    }
}

onMounted(async() => {
    await listen<ScanStateEvent>("scan_state", (event) => {
        console.log(event)
        notify({
            type: "info",
            title: `Scan ${event.payload.state}`,
            text: event.payload.state === "complete" 
                ? `${event.payload.total} files scanned, ${event.payload.added} new addons found, ${event.payload.failed} errors` 
                : ""
        })
        // Trigger refresh if scan complete
        if(event.payload.state == "complete") {
            onRefreshRequest()
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
})
</script>

<style>
.section-component {
  height: 720px !important;
  overflow: auto !important;
}
</style>