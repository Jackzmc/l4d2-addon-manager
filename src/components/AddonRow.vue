<template>
<tr>
    <td>
        <input type="checkbox" class="checkbox" @change="emit('select')" :checked="selected" />
    </td>
    <td>
        <a @click="showDetails">{{ entry.addon.title }}</a>
        <div class="tags" v-if="entry.tags.length > 0">
            <span class="tag" v-for="tag in entry.tags" :key="tag">{{ tag }}</span>
        </div>
    </td>
    <td>{{ formatSize(entry.addon.file_size) }}</td>
    <td><span class="tags" v-if="flags.length > 0">
        <span class="tag is-sucess" v-for="flag in flags" :key="flag">{{ flag }}</span>
    </span></td>
</tr>
</template>


<script setup lang="ts">
import { computed } from 'vue';
import { formatSize } from '../js/utils.ts';
import { AddonEntry } from '../types/Addon.ts';
import { getAddonContents } from '../js/app.ts';

const emit = defineEmits(["select", "showDetails"])

const flags = computed(() => {
    return getAddonContents(props.entry.addon.flags)
})

const props = defineProps<{
    entry: AddonEntry,
    selected: boolean
}>()

function showDetails() {
    emit("showDetails")
}
</script>