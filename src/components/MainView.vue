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
import { ScanResult, ScanResultMessage, ScanState } from '../types/App.ts';
import { listen } from '@tauri-apps/api/event';

const view = ref()


// If a refresh is requested (from sidebar), tell child to refresh, if they can
function onRefreshRequest() {
    if(view.value.refresh) {
        view.value.refresh()
    }
}

onMounted(async() => {
    await listen<ScanState>("scan_state", (event) => {
        notify({
            type: "info",
            title: `Scan ${event.payload}`,
        })
        // Trigger refresh if scan complete
        if(event.payload == "complete") {
            onRefreshRequest()
        }
    })

    await listen<ScanResult>("scan_result", (event) => {
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