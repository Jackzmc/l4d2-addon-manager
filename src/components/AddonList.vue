<template>
<table class="table is-fullwidth has-sticky-header mb-4">
    <thead>
        <tr>
            <td colspan="4">
                <div class="level">
                    <div class="level-left">
                        <template v-if="selectedCount > 0">
                            <slot name="select-buttons" :selected="selectedAddons" />
                        </template>
                    </div>
                    <div class="level-right">
                    </div>
                </div>
            </td>
        </tr>
        <tr>
            <th>
                <input type="checkbox" class="checkbox large" @input="toggleSelectAll" />
            </th>
            <th>Addon</th>
            <th style="min-width:8em">Size</th>
        </tr>
    </thead>
    <tbody>
        <AddonRow v-for="entry in props.addons" :key="entry.addon.filename" 
            :entry="entry" 
            :selected="isSelected(entry)"
            :workshop="workshop"
            @show-details="setDetailAddon(entry)"
            @select="setSelected(entry, !isSelected(entry))"
        />
    </tbody>
</table>

<ModalCard v-if="selectedEntry" :title="selectedEntry.addon.title" active @close="setDetailAddon(null)">
    <AddonInfoTable :entry="selectedEntry" />
    <template #footer>
        <div class="buttons">
            <!-- <button class="button" @click="selectedEntry = null">Close</button> -->
            <button class="button is-link">Disable Addon</button>
            <button class="button is-link is-outlined">Enable Addon</button>
            <button class="button is-danger is-outlined">Delete</button>
        </div>
    </template>
</ModalCard>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { AddonEntry } from '../types/Addon.ts';
import AddonRow from './AddonRow.vue';
import ModalCard from './ModalCard.vue';
import AddonInfoTable from './AddonInfoTable.vue';

const emit = defineEmits(["refresh"])

const props = defineProps<{
    addons: AddonEntry[],
    workshop?: boolean
}>()

const selected = ref<Record<string, boolean>>({})
const selectedEntry = ref<AddonEntry|null>(null)

const selectedCount = computed(() => {
    return selectedAddons.value.length
})
const selectedAddons = computed(() => {
    return Object.entries(selected.value)
        .filter(([, val]) => val)
        .map(([key]) => key)
})
function setDetailAddon(entry: AddonEntry | null) {
    selectedEntry.value = entry
}
function setSelected(entry: AddonEntry, value: boolean) {
    selected.value[entry.addon.filename] = value
}
function isSelected(entry: AddonEntry): boolean {
    return !!selected.value[entry.addon.filename]
}
function toggleSelectAll(event: InputEvent) {
    const state = (event.target as HTMLInputElement).checked
    const val: Record<string, boolean> = {}
    for(const entry of props.addons) {
        val[entry.addon.filename] = state
    }
    selected.value = val
}
</script>