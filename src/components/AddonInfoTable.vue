<template>
<table class="table is-fullwidth">
    <tbody>
        <tr>
            <td style="vertical-align: middle"><b>State</b></td>
            <td v-if="props.entry.enabled" class="">
                <span class="tag is-success has-text-white is-clickable has-tooltip-bottom has-tooltip-danger" 
                    data-tooltip="Click to disable addon"
                    @click="emit('set-state', false)"
                >
                    <Icon icon="check-circle-solid">Enabled</Icon>
                </span>
            </td>
            <td v-else-if="props.entry.enabled === false">
                <span class="tag is-danger has-text-white is-clickable has-tooltip-bottom has-tooltip-success" 
                    data-tooltip="Click to enable addon"
                    @click="emit('set-state', true)"
                >
                    <Icon icon="xmark-circle-solid">Not Enabled</Icon>
                </span>
            </td>
            <td v-else><em>Unknown</em></td>
        </tr>
        <tr>
            <td><b>Filename</b></td>
            <td v-if="props.entry.info.filename">
                <code>{{ props.entry.info.filename }}</code>
                <span class="ml-2"><a @click="openFile">(Open file)</a></span>
            </td>
            <td v-else class="has-text-danger">
                Missing <em>(cannot find file, was it renamed or deleted?)</em>
                <!-- <a>(Select file)</a> -->
            </td>
        </tr>
        <tr>
            <td><b>Author</b></td>
            <td>{{ props.entry.info.author }}</td>
        </tr>
        <tr>
            <td><b>Version</b></td>
            <td>{{ props.entry.info.version }}</td>
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
                    <span class="tag has-background-primary-light" v-for="tag in flags" :key="tag">{{ tag }}</span>
                </div>
            </td>
        </tr>
        <tr v-if="!workshop">
            <td><b>My Tags</b></td>
            <td>
                <div class="tags mb-2">
                    <span v-for="tag in props.entry.tags" :key="tag" 
                        class="tag has-tooltip-danger" data-tooltip="click to delete tag"
                        @click="onDelTagPressed(tag)"
                    >tag:{{ tag }}</span>
                    <span class="button is-link is-small" @click="onAddTagPressed">+ Add Tag</span>
                </div>
            </td>
        </tr>
        <tr>
            <td><b>Workshop ID</b></td>
            <td v-if="props.entry.info.workshop_id">
                <code>{{ props.entry.info.workshop_id}}</code>
                <a class="tag is-link is-medium ml-3 has-tooltip-link" data-tooltip="Open steam workshop page (in main browser)"
                    :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + props.entry.info.workshop_id" target="_blank"
                >
                    <Icon icon="open-new-window" text-left="Steam Workshop"></Icon>
                </a>
            </td>
            <td v-else><em>not set</em> <!--<a>(Click to set)</a>--></td>
        </tr>
        <tr v-if="!workshop">
            <td><b>SHA256 Hash</b></td>
            <td><code>{{ props.entry.id }}</code></td>
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
import { addTag, removeTag, showAddonInFolder } from '../js/tauri.ts';
import Icon from './Icon.vue';

const emit = defineEmits(["refresh", "set-state"])

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
    return props.workshop ? props.entry.workshop?.tags.split(",") : getAddonContents(props.entry.info.flags)
})

async function onAddTagPressed() {
    const tagValue = prompt("Enter tag")
    if(tagValue) {
        await addTag(props.entry.id, tagValue)
        emit("refresh")
    }
}
async function onDelTagPressed(tag: string) {
    await removeTag(props.entry.id, tag)
    emit("refresh")
}

async function openFile() {
    await showAddonInFolder(props.entry.info.filename, false)
}
</script>