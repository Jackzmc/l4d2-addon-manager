import { AddonEntry } from '../../types/Addon';
<template>
<ModalCard :title="props.entry.addon.title" active @close="emit('close')">
    <AddonInfoTable :entry="entry" />
    <template #footer>
        <div class="buttons" v-if="props.entry.addon.filename">
            <!-- <button class="button" @click="selectedEntry = null">Close</button> -->
            <button @click="onSetState(false)" class="button is-link">Disable Addon</button>
            <button @click="onSetState(true)" class="button is-link is-outlined">Enable Addon</button>
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
import { deleteAddons, setAddonState } from '../../js/tauri.ts';
import { AddonEntry } from '../../types/Addon.ts';
import AddonInfoTable from '../AddonInfoTable.vue';
import ModalCard from '../ModalCard.vue';

const emit = defineEmits(["close", "set-state", "delete"])

const props = defineProps<{
    entry: AddonEntry
}>()

async function onSetState(state: boolean) {
    setAddonState([props.entry.addon.filename], state)
}

async function onDeletePressed() {
    await deleteAddons([props.entry.addon.filename])
}

</script>