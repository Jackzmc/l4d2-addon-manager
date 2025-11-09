<template>
<tr class="row">
    <td>
        <input v-if="props.entry.addon.filename" type="checkbox" class="checkbox large" @change="emit('select')" :checked="selected" />
    </td>
    <td>
        <a @click="showDetails">{{ entry.addon.title }}</a>
        <div class="tags" v-if="tags.length > 0">
            <a class="tag has-background-primary-light" v-for="tag in tags" :key="tag" @click="selectTag(tag)">{{ tag }}</a>
        </div>
    </td>
    <td>{{ formatSize(entry.addon.file_size) }}</td>
    <!-- <td v-if="!workshop"><span class="tags" v-if="tags.length > 0">
        <span class="tag is-sucess" v-for="flag in tags" :key="flag">{{ flag }}</span>
    </span></td> -->
</tr>
</template>


<script setup lang="ts">
import { computed } from 'vue';
import { formatSize } from '../js/utils.ts';
import { AddonEntry } from '../types/Addon.ts';
import { getAddonContents } from '../js/app.ts';

const emit = defineEmits(["select", "showDetails", "selectTag"])

const tags = computed(() => {
    return props.workshop ? props.entry.tags : getAddonContents(props.entry.addon.flags)
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