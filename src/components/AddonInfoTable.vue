<template>
<table class="table is-fullwidth">
    <thead>
        <tr>
            <th>Name</th>
            <th>Value</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><b>State</b></td>
            <td>️✅ Enabled</td>
        </tr>
        <tr>
            <td><b>Filename</b></td>
            <td v-if="props.entry.addon.filename">{{ props.entry.addon.filename }}</td>
            <td v-else class="has-text-danger">
                Missing
                <!-- <a>(Select file)</a> -->
            </td>
        </tr>
        <tr>
            <td><b>Created</b></td>
            <td>{{ createdAt.toLocaleString() }} <em>({{ createdAtRel }})</em></td>
        </tr>
        <tr>
            <td><b>Last Updated</b></td>
            <td>{{ updatedAt.toLocaleString() }} <em>({{ updatedAtRel }})</em></td>
        </tr>
        <tr>
            <td><b>File Size</b></td>
            <td>{{ formatSize(props.entry.addon.file_size) }}</td>
        </tr>
        <tr>
            <td><b>Author</b></td>
            <td>{{ props.entry.addon.author }}</td>
        </tr>
        <tr>
            <td><b>Version</b></td>
            <td>{{ props.entry.addon.version }}</td>
        </tr>
        <!-- <tr>
            <td><b>SHA-256 Hash</b></td>
            <td>{{ props.entry.hash }}</td>
        </tr> -->
        <tr v-if="props.entry.addon.chapter_ids">
            <td><b>Chapter Ids</b></td>
            <td>
                <div class="tags">
                    <span class="tag" v-for="tag in props.entry.addon.chapter_ids.split(',')" :key="tag">{{ tag }}</span>
                </div>
            </td>
        </tr>
        <tr>
            <td><b>Content</b></td>
            <td>
                <div class="tags">
                    <span class="tag" v-for="tag in flags" :key="tag">{{ tag }}</span>
                </div>
            </td>
        </tr>
        <tr>
            <td><b>Tags</b></td>
            <td>
                <div class="tags">
                    <span class="tag" v-for="tag in props.entry.tags" :key="tag">{{ tag }}</span>
                </div>
            </td>
        </tr>
        <tr>
            <td><b>Workshop ID</b></td>
            <td v-if="props.entry.addon.workshop_id">
                {{ props.entry.addon.workshop_id}}
                <a :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + props.entry.addon.workshop_id" target="_blank">
                    (View on Steam Workshop)
                </a>
            </td>
            <td v-else><em>not set</em> <a>(Click to set)</a></td>
        </tr>
    </tbody>
</table>
</template>


<script setup lang="ts">
import { computed } from 'vue';
import { formatSize } from '../js/utils.ts';
import { AddonEntry } from '../types/Addon.ts';
import { getRelDate } from '../js/utils';
import { getAddonContents } from '../js/app.ts';

const createdAt = computed(() => {
    return new Date(props.entry.addon.created_at)
})
const createdAtRel = computed(() => {
    return getRelDate(createdAt.value)
})
const updatedAt = computed(() => {
    return new Date(props.entry.addon.updated_at)
})
const updatedAtRel = computed(() => {
    return getRelDate(updatedAt.value)
})

const flags = computed(() => {
    return getAddonContents(props.entry.addon.flags)
})

const props = defineProps<{
    entry: AddonEntry
}>()
</script>