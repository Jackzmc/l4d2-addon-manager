import { AddonEntry } from '../../types/Addon';
<template>
<ModalCard :title="props.entry.info.title" active @close="emit('close')">
    <AddonInfoTable :entry="entry" />
    <template #footer>
        <div class="buttons" v-if="props.entry.info.filename">
            <!-- <button class="button" @click="selectedEntry = null">Close</button> -->
            <button v-if="props.entry.enabled" @click="onSetState(false)" class="button is-link  is-outlined">Disable Addon</button>
            <button v-else-if="props.entry.enabled === false" @click="onSetState(true)" class="button is-link">Enable Addon</button>
            <button @click="onDeletePressed" class="button is-danger is-outlined">Delete</button>
        </div>
        <span v-else>
            File has been moved or deleted, no actions available
            <!-- TODO: add delete entry button -->
        </span>
    </template>
</ModalCard>
</template>

<script setup lang="ts">
import { confirm } from '@tauri-apps/plugin-dialog';
import { deleteAddons, setAddonState } from '../../js/tauri.ts';
import { AddonEntry } from '../../types/Addon.ts';
import AddonInfoTable from '../AddonInfoTable.vue';
import ModalCard from '../ModalCard.vue';

const emit = defineEmits(["close", "refresh", "set-state", "delete"])

const props = defineProps<{
    entry: AddonEntry
}>()

async function onSetState(state: boolean) {
    await setAddonState([props.entry.info.filename], state)
    emit("refresh")
}

async function onDeletePressed() {
    if(await confirm(`Are you sure you want to delete "${props.entry.info.title}"? It will be moved to trash and removed from the manager.`, { title: "Confirm Deletion", okLabel: "Delete" })) {
        await deleteAddons([props.entry.info.filename])
        emit("refresh")
        emit("close")
    }

}

</script>