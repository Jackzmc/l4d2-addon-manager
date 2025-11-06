<template>
  {{ size }}
  <notifications position="bottom right" :speed="5000" />
  <router-view />

</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { onBeforeMount, ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter()

const size = ref([0, 0])

onBeforeMount(async() => {
  console.debug("listening")
  await listen<string>("set_route", (event) => {
    console.log("set_route", event)
    const route = event.payload
    router.push(route)
  })
})

  size.value = [window.innerWidth, window.innerHeight]
window.addEventListener("resize", () => {
  size.value = [window.innerWidth, window.innerHeight]
})
</script>

<style>
html, body {
  overflow-y: hidden !important;
  background-color: rgba(255, 255, 255, 0.667);
  height: 100%;
}
</style>
