<template>
<tr>
    <td>{{ entry.addon.filename }}</td>
    <td>{{ formatSize(entry.addon.file_size) }}</td>
    <td><span class="tags" v-if="flags.length > 0">
        <span class="tag is-sucess" v-for="flag in flags" :key="flag">{{ flag }}</span>
    </span></td>
    <td><span class="tags" v-if="entry.tags.length > 0">
        <span class="tag" v-for="tag in entry.tags" :key="tag">{{ tag }}</span>
    </span></td>
</tr>
</template>


<script setup lang="ts">
import { computed } from 'vue';
import { formatSize } from '../js/utils.ts';
import { AddonEntry, AddonFlags } from '../types/Addon.ts';

const FLAG_TAG_NAMES: Record<number, string | undefined> = {
    [AddonFlags.Campaign]: 'Map'
}

const flags = computed(() => {
    const tags = []
    for(const [flag, name] of Object.entries(FLAG_TAG_NAMES)) {
        if(props.entry.addon.flags & Number(flag)) {
            tags.push(name)
        }
    }
    return tags
})

const props = defineProps<{
    entry: AddonEntry
}>()
</script>