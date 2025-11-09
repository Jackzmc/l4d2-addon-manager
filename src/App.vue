<template>
  <notifications position="bottom right" :speed="5000" />
  <router-view @init="onInit" :static-data="staticData" :config="configData" />

</template>

<script setup lang="ts">
import { onBeforeMount, ref } from 'vue';
import { InitAppData, StaticAppData, AppConfig } from './types/App.ts';

const staticData = ref<StaticAppData>()
const configData = ref<AppConfig>()
function onInit(init: InitAppData) {
  staticData.value = init.data
  configData.value = init.config
}
</script>

<style>
html, body {
  overflow-y: hidden !important;
  background-color: rgba(255, 255, 255, 0.667);
  height: 100%;
}
</style>
