<template>
<table class="table is-fullwidth">
    <thead>
        <tr>
            <th>File Name</th>
            <th>File Size</th>
            <th>Info</th>
            <th>Tags</th>
        </tr>
    </thead>
    <tbody>
        <tr v-for="entry in props.addons" :key="entry.addon.filename">
            <td>{{ entry.addon.filename }}</td>
            <td>{{ entry.addon.file_size }}</td>
            <td>{{ entry.addon.flags }}</td>
            <td><span class="tags" v-if="entry.tags.length > 0">
                <span class="tag" v-for="tag in entry.tags" :key="tag">{{ tag }}</span>
            </span></td>
        </tr>
    </tbody>
</table>
</template>

<script setup lang="ts">
import { AddonEntry } from '../types/Addon.ts';

const props = defineProps<{
    addons: AddonEntry[]
}>()
</script>