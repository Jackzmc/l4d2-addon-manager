<template>
    <AddonList workshop :addons="addons" @refresh="refresh">
        <template #select-buttons>
            <button class="level-item button is-warning" @click="refresh">Move to managed</button>
            <button class="level-item button is-link">Unsubscribe</button>
        </template>
    </AddonList>
    <p class="has-text-centered my-6" v-if="addons.length === 0">
        No addons found
    </p>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { AddonEntry } from '../types/Addon.ts';
import { listAddons } from '../js/tauri.ts';
import AddonList from '../components/AddonList.vue';

const addons = ref<AddonEntry[]>([])

async function refresh() {
    addons.value = await listAddons(true)
    console.debug("got addons", addons.value)
}

onMounted(() => {
    refresh()
})

defineExpose({
    refresh
})
</script>