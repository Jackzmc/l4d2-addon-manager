<template>
    managed
    <AddonList :addons="addons" />
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { Addon } from '../types/Addon.ts';
import { getManagedAddons } from '../js/tauri.ts';

const addons = ref<Addon[]>([])

async function getAddons() {
    addons.value = await getManagedAddons()
}

onMounted(() => {
    getAddons()
})
</script>