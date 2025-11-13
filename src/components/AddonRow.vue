<template>
<tr :class="['row', 
    {
        'has-background-danger-light': !props.entry.info.filename, 
        'selected': selected, 'selecting': isSelecting,
        'is-clickable': isSelecting,
        'is-unselectable': isSelecting
}]">
    <td>
        <input v-if="props.entry.info.filename" type="checkbox" class="checkbox extra-large" @change="emit('select')" :checked="selected" />
    </td>
    <td>
        <a @click="showDetails">
            <strong v-if="entry.enabled">{{ entry.info.title }}</strong>
            <span v-else class="has-text-black">{{ entry.info.title }}</span>
        </a>
        <div class="tags mb-2" v-if="tags.length > 0 || props.entry.tags.length > 0">
            <a v-for="tag in tags" :key="tag" class="tag has-background-primary-light" 
                @click="selectTag(tag)"
            >
                {{ tag }}
            </a>
            <template v-if="!props.workshop">
            <a v-for="tag in props.entry.tags" class="tag" :key="tag" 
                @click="selectTag(tag)"
            >tag:{{ tag }}</a>
            </template>
        </div>
    </td>
    <td>{{ formatSize(entry.info.file_size) }}</td>
    <td>{{ getRelDate(new Date(entry.info.updated_at)) }}</td>
    <!-- <td>{{ props.entry.tags }}</td> -->
    <!-- <td v-if="!workshop"><span class="tags" v-if="tags.length > 0">
        <span class="tag is-sucess" v-for="flag in tags" :key="flag">{{ flag }}</span>
    </span></td> -->
</tr>
</template>


<script setup lang="ts">
import { computed } from 'vue';
import { formatSize, getRelDate } from '../js/utils.ts';
import { AddonEntry } from '../types/Addon.ts';
import { getAddonContents } from '../js/app.ts';

const emit = defineEmits(["select", "showDetails", "selectTag"])

const tags = computed(() => {
    return props.workshop ? props.entry.tags : getAddonContents(props.entry.info.flags)
})

const props = defineProps<{
    entry: AddonEntry,
    workshop?: boolean,
    selected: boolean,

    isSelecting?: boolean
}>()

function showDetails() {
    if(props.isSelecting) return // prevent accidental title clicks when selecting
    emit("showDetails")
}
function selectTag(tag: string) {
    if(props.isSelecting) return // prevent accidental title clicks when selecting
    emit("selectTag", tag)
}
</script>

<style scoped>
.row.selecting {
    opacity: 0.6;
}

.row.selecting.selected {
    opacity: 1;
    background-color: rgb(232, 246, 252);
}
.row.selecting.selected:hover {
    opacity: 1;
    background-color: rgb(204, 240, 255);
}
</style>