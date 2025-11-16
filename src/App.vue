<template>
  <notifications position="bottom right" :duration="5000" />
  <router-view @init="onInit" :static-data="staticData" :config="configData" />
</template>

<script setup lang="ts">
import { onBeforeMount, onMounted, ref } from 'vue';
import { InitAppData, StaticAppData, AppConfig } from './types/App.ts';
import { listen } from '@tauri-apps/api/event';

const staticData = ref<StaticAppData>()
const configData = ref<AppConfig>()
function onInit(init: InitAppData) {
  staticData.value = init.data
  configData.value = init.config
}

onMounted(async() => {
  await listen<AppConfig>("config_changed", (event) => {
    console.debug(event.event)
    configData.value = event.payload
  })
})
</script>

<style>
html, body, #app {
  background-color: rgba(255, 255, 255, 0.667);
  height: 100%;
  overflow: hidden;
}
</style>
