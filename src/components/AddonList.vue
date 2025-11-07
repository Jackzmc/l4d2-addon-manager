<template>
<table class="table is-fullwidth has-sticky-header mb-4">
    <thead>
        <tr>
            <td colspan="5">
                <div class="level">
                    <div class="level-left">
                        <template v-if="selectedCount > 0">
                            <button class="level-item button" @click="refresh">Disable</button>
                            <button class="level-item button" @click="refresh">Delete</button>
                        </template>
                    </div>
                    <div class="level-right">
                        <button class="level-item button is-link" @click="refresh">Refresh</button>
                        <button class="level-item button is-link" @click="startScan">Rescan</button>
                    </div>
                </div>
            </td>
        </tr>
        <tr>
            <th>
                <input type="checkbox" class="checkbox large" @input="toggleSelectAll" />
            </th>
            <th>Addon</th>
            <th>Size</th>
            <th>Content</th>
        </tr>
    </thead>
    <tbody>
        <AddonRow v-for="entry in props.addons" :key="entry.addon.filename" 
            :entry="entry" 
            :selected="isSelected(entry)"
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
import { notify } from '@kyvg/vue3-notification';
import { scanAddons } from '../js/tauri.ts';
import { AddonEntry } from '../types/Addon.ts';
import AddonRow from './AddonRow.vue';
import ModalCard from './ModalCard.vue';
import AddonInfoTable from './AddonInfoTable.vue';

const emit = defineEmits(["refresh"])

const props = defineProps<{
    addons: AddonEntry[]
}>()

const selected = ref<Record<string, boolean>>({})
const selectedEntry = ref<AddonEntry|null>(null)

const selectedCount = computed(() => {
    return Object.values(selected.value).filter(selected => selected).length
})

function refresh() {
    emit("refresh")
}
async function startScan() {
    try {
        await scanAddons()
        notify({
            type: "info",
            title: "Scan started",
            text: "Scan has started in the background. This may take some time."
        })
    } catch(err: any) {
        notify({
            type: "error",
            title: "Scan failed",
            text: err.message ?? err
        })
    }
}
function setDetailAddon(entry: AddonEntry | null) {
    selectedEntry.value = entry
    console.debug("selected", entry?.addon.filename)
}
function setSelected(entry: AddonEntry, value: boolean) {
    selected.value[entry.addon.title] = value
    console.log("setSelected", entry.addon.title, value)
}
function isSelected(entry: AddonEntry): boolean {
    return !!selected.value[entry.addon.title]
}
function toggleSelectAll(event: InputEvent) {
    const state = (event.target as HTMLInputElement).checked
    console.debug("select", event)
    const val: Record<string, boolean> = {}
    // TODO: unselect
    for(const entry of props.addons) {
        val[entry.addon.title] = state
    }
    selected.value = val
    console.debug("selected all")
}
</script>