<template>
    <div class="columns">
        <div class="column is-3" >
            <Sidebar />
        </div>
        <main class="column mt-3 section-component">
            <router-view />
        </main>
    </div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { onMounted } from 'vue';
import { scanAddons } from '../js/tauri.ts';

onMounted(() => {
    scanAddons(false)
})
</script>

<style>
.section-component {
  height: 720px !important;
  overflow: auto !important;
}
</style>