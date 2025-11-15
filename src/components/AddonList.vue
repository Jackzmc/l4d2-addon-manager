<template>
<table class="table is-fullwidth has-sticky-header mb-4 is-hoverable">
    <thead>
        <tr>
            <td colspan="4">
                <div class="level mb-2">
                    <div class="level-left">
                        <template v-if="selectedCount > 0">
                            <slot name="select-buttons" :selected="selectedAddons" />
                        </template>
                    </div>
                    <div class="level-right">
                        <Field icon-right="iconoir:search">
                            <input style="width: 400px" type="text" class="input" placeholder="Search for an item" v-model="query" />
                        </Field>
                    </div>
                </div>
            </td>
        </tr>
        <tr>
            <th data-tooltip="Select all">
                <input type="checkbox" class="checkbox extra-large" v-model="selectAll" @input="toggleSelectAll"/>
            </th>
            <SortableColumnHeader @sort="setSort" label="Addon Name" field="title" :sort="sort" />
            <SortableColumnHeader @sort="setSort" label="Size" field="file_size" :sort="sort" style="min-width:8em" />
            <SortableColumnHeader @sort="setSort" label="Updated" field="updated_at" :sort="sort" style="min-width:8em" />
        </tr>
    </thead>
    <tbody>
        <AddonRow v-for="entry in filteredAddons" :key="entry.id" 
            :entry="entry" 
            :selected="isSelected(entry)"
            :workshop="workshop"
            :is-selecting="isSelecting"
            @show-details="setDetailAddon(entry)"
            @select="setSelected(entry, !isSelected(entry))"
            @select-tag="onTagSelected"

            @click="onEntryClicked(entry)"
        />
    </tbody>
</table>

<AddonInfoModal workshop v-if="selectedEntry" :entry="selectedEntry" @close="setDetailAddon(null)" @refresh="onRefresh" />
</template>

<style>

</style>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { AddonEntry } from '../types/Addon.ts';
import AddonRow from './AddonRow.vue';
import { getAddonContents } from '../js/app.ts';
import AddonInfoModal from './modals/AddonInfoModal.vue';
import Field from './Field.vue';
import Icon from './Icon.vue';
import SortableColumnHeader, { SelectedSort } from './SortableColumnHeader.vue';

const SORT_ICONS = ["iconoir:sort", "iconoir:sort-down", "iconoir:sort-up"]

const emit = defineEmits(["refresh"])

const props = defineProps<{
    addons: AddonEntry[],
    workshop?: boolean
    sort: SelectedSort
}>()

const selectAll = ref(false)
const selected = ref<Record<string, boolean>>({})
const selectedEntry = ref<AddonEntry|null>(null)

const query = ref<string>("")

const selectedCount = computed(() => {
    return selectedAddons.value.length
})
const isSelecting = computed(() => selectedCount.value > 0)
const selectedAddons = computed(() => {
    return Object.entries(selected.value)
        .filter(([, val]) => val)
        .map(([key]) => key)
})
function setDetailAddon(entry: AddonEntry | null) {
    selectedEntry.value = entry
}
function setSelected(entry: AddonEntry, value: boolean) {
    selected.value[entry.info.filename] = value
}
function isSelected(entry: AddonEntry): boolean {
    return !!selected.value[entry.info.filename]
}
function toggleSelectAll(event: InputEvent) {
    const state = (event.target as HTMLInputElement).checked
    const val: Record<string, boolean> = {}
    for(const entry of props.addons) {
        val[entry.info.filename] = state
    }
    selected.value = val
}
function setSort(field: string, descending = false) {
    if(props.sort?.field === field) {
        // Flip order if same field
        emit("refresh", { field: props.sort.field, descending: !props.sort.descending })
    } else {
        emit("refresh", { field, descending })
    }
}

function onTagSelected(tag: string) {
    if(query.value.length > 0) {
        query.value += " "
    }
    query.value += `#${tag}`
}

function onRefresh() {
    emit('refresh')
    if(selectedEntry.value) {
        // Update the selected addon modal with the updated data from the list
        console.debug('replaced selectedEntry', selectedEntry.value?.id)
    }   
}

function onEntryClicked(entry: AddonEntry) {
    // only select if we already started selecting
    if(isSelecting.value) {
        setSelected(entry, !isSelected(entry))
    }
}

function clearSelection() {
    for(const entry of props.addons) {
        selected.value[entry.info.filename] = false
    }
    selectAll.value = false
}

watch(() => props.addons, () => {
    if(selectedEntry.value) {
        selectedEntry.value = props.addons.find(entry => entry.id === selectedEntry.value!.id) ?? null
    }
})

const filteredAddons = computed(() => {
    if(query.value === "") return props.addons
    const q = query.value.toLocaleLowerCase()
    return props.addons.filter(entry => {
        return entry.info.title.toLocaleLowerCase().includes(q)
            || entry.info.filename.toLocaleLowerCase().includes(q)
            || entry.info.tagline?.toLocaleLowerCase().includes(q)
            // expensive but oh well seems fine
            || entry.tags.some(tag => queryTags.value.includes(tag.toLocaleLowerCase()))
            || getAddonContents(entry.info.flags).some(tag => queryTags.value.includes(tag.toLocaleLowerCase()))
    })
})
const queryTags = computed(() => {
    if(query.value === "") return []
    const tags = []
    const split = query.value.split(" ")
    for(const piece of split) {
        if(piece.startsWith("#")) {
            tags.push(piece.substring(1).toLocaleLowerCase())
        }
    }
    return tags
})

defineExpose({ clearSelection })
</script>