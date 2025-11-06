<template>
  <notifications position="bottom right" :speed="5000" />
  <router-view />

</template>

<script setup lang="ts">
import { notify } from '@kyvg/vue3-notification';
import { listen } from '@tauri-apps/api/event';
import { onBeforeMount, ref } from 'vue';
import { useRouter } from 'vue-router';
import { ScanResult, ScanResultMessage, ScanState } from './types/App.ts';

const router = useRouter()

onBeforeMount(async() => {
  console.debug("listening")
  await listen<string>("set_route", (event) => {
    console.log("set_route", event)
    const route = event.payload
    router.push(route)
  })

  await listen<ScanState>("scan_state", (event) => {
    notify({
      type: "info",
      title: `Scan ${event.payload}`,
    })
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
html, body {
  overflow-y: hidden !important;
  background-color: rgba(255, 255, 255, 0.667);
  height: 100%;
}
</style>
