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
            <td v-if="props.entry.enabled">️✅ Enabled</td>
            <td v-else-if="props.entry.enabled === false">❌ Disabled</td>
            <td v-else><em>Unknown</em></td>
        </tr>
        <tr>
            <td><b>Filename</b></td>
            <td v-if="props.entry.info.filename">
                <code>{{ props.entry.info.filename }}</code>
            </td>
            <td v-else class="has-text-danger">
                Missing <em>(cannot find file, was it renamed or deleted?)</em>
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
            <td>{{ formatSize(props.entry.info.file_size) }}</td>
        </tr>
        <tr>
            <td><b>Author</b></td>
            <td>{{ props.entry.info.author }}</td>
        </tr>
        <tr>
            <td><b>Version</b></td>
            <td>{{ props.entry.info.version }}</td>
        </tr>
        <tr v-if="!workshop">
            <td><b>SHA256 Hash</b></td>
            <td>{{ props.entry.id }}</td>
        </tr>
        <tr v-if="props.entry.info.chapter_ids">
            <td><b>Chapter Ids</b></td>
            <td>
                <div class="tags">
                    <span class="tag" v-for="tag in props.entry.info.chapter_ids.split(',')" :key="tag">{{ tag }}</span>
                </div>
            </td>
        </tr>
        <tr>
            <td><b>Content Types</b></td>
            <td>
                <div class="tags">
                    <span class="tag" v-for="tag in flags" :key="tag">{{ tag }}</span>
                </div>
            </td>
        </tr>
        <tr>
            <td><b>My Tags</b></td>
            <td>
                <div class="tags mb-2" v-if="!workshop">
                    <span class="tag" v-for="tag in props.entry.tags" :key="tag">tag:{{ tag }}</span>
                    <span class="button is-link is-small" @click="onAddTagPressed">+ Add Tag</span>
                </div>
            </td>
        </tr>
        <tr>
            <td><b>Workshop ID</b></td>
            <td v-if="props.entry.info.workshop_id">
                <code>{{ props.entry.info.workshop_id}}</code>
                <a :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + props.entry.info.workshop_id" target="_blank">
                    (View on Steam Workshop)
                </a>
            </td>
            <td v-else><em>not set</em> <!--<a>(Click to set)</a>--></td>
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
import { addTag } from '../js/tauri.ts';

const emit = defineEmits(["refresh"])

const props = defineProps<{
    entry: AddonEntry,
    workshop?: boolean
}>()

const createdAt = computed(() => {
    return new Date(props.entry.info.created_at)
})
const createdAtRel = computed(() => {
    return getRelDate(createdAt.value)
})
const updatedAt = computed(() => {
    return new Date(props.entry.info.updated_at)
})
const updatedAtRel = computed(() => {
    return getRelDate(updatedAt.value)
})
const flags = computed(() => {
    return getAddonContents(props.entry.info.flags)
})

async function onAddTagPressed() {
    const tagValue = prompt("Enter tag")
    if(tagValue) {
        await addTag(props.entry.id, tagValue)
        emit("refresh")
    }
}
</script>