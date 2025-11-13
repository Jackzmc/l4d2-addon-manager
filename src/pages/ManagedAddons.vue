<template>
<div>
    <AddonList :addons="addons" @refresh="refresh" ref="list">
        <template #select-buttons="{selected}">
            <button class="level-item button " @click="onClearPressed">
                <Icon icon="erase">Clear Selection</Icon>
            </button>
            <button class="level-item button is-link has-tooltip-right" data-tooltip="Enable all selected addons" @click="onSetStatePressed(selected, true)">Enable</button>
            <button class="level-item button is-link is-outlined has-tooltip-right" data-tooltip="Enable all selected addons" @click="onSetStatePressed(selected, false)">Disable</button>
            <button class="level-item button is-danger has-tooltip-right has-tooltip-danger" data-tooltip="Delete all selected addons" @click="onDeletePressed(selected)">Delete</button>
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
import { confirm } from '@tauri-apps/plugin-dialog';
import Icon from '../components/Icon.vue';

const list = ref()
const addons = ref<AddonEntry[]>([])

async function refresh() {
    addons.value = await listAddons(false)
    console.debug("got addons", addons.value)
}

function onClearPressed() {
    list.value.clearSelection()
}
 
async function onSetStatePressed(filenames: string[], state: boolean) {
    await setAddonState(filenames, state)
}

async function onDeletePressed(filenames: string[]) {
    if(await confirm(`Are you sure you want to delete these addons? They will be moved to trash and removed from the manager.`, { title: "Confirm Deletion", okLabel: "Delete" })) {
        await deleteAddons(filenames)
        await refresh()
    }
}

onMounted(() => {
    refresh()
})

defineExpose({
    refresh
})
</script>