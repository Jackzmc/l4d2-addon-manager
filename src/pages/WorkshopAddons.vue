<template>
    <AddonList workshop :addons="addons" @refresh="refresh">
        <template #select-buttons="{selected}">
            <button class="level-item button is-warning" @click="onMigratePressed(selected)">Move to managed addons</button>
            <!-- TODO: support-->
            <!-- <button class="level-item button is-link" @click="onUnsubscribe">Unsubscribe</button> -->
        </template>
    </AddonList>
    <p class="has-text-centered my-6" v-if="addons.length === 0">
        No addons found
    </p>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { AddonEntry } from '../types/Addon.ts';
import { listAddons, migrateWorkshopAddons, unsubscribeAddons } from '../js/tauri.ts';
import AddonList from '../components/AddonList.vue';

const addons = ref<AddonEntry[]>([])

async function refresh() {
    addons.value = await listAddons(true)
    console.debug("got addons", addons.value)
}

/** convert '######.vpk' -> ##### */
function filenamesToWorkshopIds(filenames: string[]): number[] {
    return filenames.map(file => Number(file.slice(0, -4)))
}

async function onMigratePressed(filenames: string[]) {
    // convert '######.vpk' -> #####
    await migrateWorkshopAddons(filenamesToWorkshopIds(filenames))
}

async function onUnsubscribePressed(filenames: string[]) {
    await unsubscribeAddons(filenamesToWorkshopIds(filenames))
}

onMounted(() => {
    refresh()
})

defineExpose({
    refresh
})
</script>