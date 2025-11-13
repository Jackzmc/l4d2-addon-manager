<template>
<tr :class="['row', {'has-background-danger-light': !props.entry.info.filename }]">
    <td>
        <input v-if="props.entry.info.filename" type="checkbox" class="checkbox extra-large" @change="emit('select')" :checked="selected" />
    </td>
    <td>
        <a @click="showDetails">
            <strong v-if="entry.enabled">{{ entry.info.title }}</strong>
            <span v-else class="has-text-black">{{ entry.info.title }}</span>
        </a>
        <div class="tags mb-2" v-if="tags.length > 0 || props.entry.tags.length > 0">
            <a class="tag has-background-primary-light" v-for="tag in tags" :key="tag" @click="selectTag(tag)">{{ tag }}</a>
            <a v-if="!props.workshop" class="tag" v-for="tag in props.entry.tags" :key="tag" @click="selectTag(tag)">tag:{{ tag }}</a>
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
    selected: boolean
}>()

function showDetails() {
    emit("showDetails")
}
function selectTag(tag: string) {
    emit("selectTag", tag)
}
</script>