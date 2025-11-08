<template>
  <notifications position="bottom right" :speed="5000" />
  <router-view @init="onInit" :static-data="data" />

</template>

<script setup lang="ts">
import { onBeforeMount, ref } from 'vue';
import { InitAppData, StaticAppData } from './types/App.ts';

const data = ref<StaticAppData>()
function onInit(init: InitAppData) {
  data.value = init.data
}
</script>

<style>
html, body {
  overflow-y: hidden !important;
  background-color: rgba(255, 255, 255, 0.667);
  height: 100%;
}
</style>
