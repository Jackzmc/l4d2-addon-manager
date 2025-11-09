<template>
<div>
    <AddonList :addons="addons" @refresh="refresh">
        <template #select-buttons="{selected}">
            <button class="level-item button is-success" @click="onSetStatePressed(selected, true)">Enable</button>
            <button class="level-item button is-warning" @click="onSetStatePressed(selected, false)">Disable</button>
            <button class="level-item button is-danger" @click="onDeletePressed(selected)">Delete</button>
        </template>
    </AddonList>
    <p class="has-text-centered my-6" v-if="addons.length === 0">
        No addons found
    </p>
</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { AddonEntry } from '../types/Addon.ts';
import { deleteAddons, setAddonState, listAddons } from '../js/tauri.ts';
import AddonList from '../components/AddonList.vue';

const addons = ref<AddonEntry[]>([])

async function refresh() {
    addons.value = await listAddons(false)
    console.debug("got addons", addons.value)
}
 
async function onSetStatePressed(filenames: string[], state: boolean) {
    await setAddonState(filenames, state)
}

async function onDeletePressed(filenames: string[]) {
    await deleteAddons(filenames)
}

onMounted(() => {
    refresh()
})

defineExpose({
    refresh
})
</script>