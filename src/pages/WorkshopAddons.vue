<template>
<div>
    <AddonList workshop :addons="addons" :sort="sort" @refresh="refresh" ref="list">
        <template #select-buttons="{selected}">
            <button class="level-item button " @click="onClearPressed">
                <Icon icon="erase">Clear Selection</Icon>
            </button>
            <button class="level-item button is-warning has-tooltip-right" 
                data-tooltip="Copies workshop files to managed addons and unsubscribes"
                @click="onMigratePressed(selected)"
            >
                Move to managed addons
            </button>
            <button v-if="config.steam_apikey" class="level-item button is-link has-tooltip-right" 
                data-tooltip="Unsubscribes addon from workshop"
                @click="onUnsubscribePressed(selected)">
                Unsubscribe
            </button>
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
import { listAddons, migrateWorkshopAddons, unsubscribeAddons } from '../js/tauri.ts';
import AddonList from '../components/AddonList.vue';
import { AppConfig } from '../types/App.ts';
import { SelectedSort } from '../components/SortableColumnHeader.vue';

const emit = defineEmits(["scan"])
const props = defineProps<{
    config: AppConfig
}>()

const list = ref()
const addons = ref<AddonEntry[]>([])
const sort = ref<SelectedSort>({ field: "title", descending: false })

async function refresh(newSort?: SelectedSort) {
    if(newSort) sort.value = newSort
    addons.value = await listAddons(true, sort.value)
    console.debug("got addons", addons.value)
}

/** convert '######.vpk' -> ##### */
function filenamesToWorkshopIds(filenames: string[]): number[] {
    return filenames.map(file => Number(file.slice(0, -4)))
}

function onClearPressed() {
    list.value.clearSelection()
}

// TODO: better solution than rescan?
async function onMigratePressed(filenames: string[]) {
    // convert '######.vpk' -> #####
    await migrateWorkshopAddons(filenamesToWorkshopIds(filenames))
    onClearPressed()  // clear selection
}

async function onUnsubscribePressed(filenames: string[]) {
    await unsubscribeAddons(filenamesToWorkshopIds(filenames))
    onClearPressed() // clear selection
}

onMounted(() => {
    refresh()
})

defineExpose({
    refresh
})
</script>