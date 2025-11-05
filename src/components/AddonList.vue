<template>
<table class="table is-fullwidth has-sticky-header">
    <thead>
        <tr>
            <th>File Name</th>
            <th>File Size</th>
            <th>Info</th>
            <th>Tags</th>
        </tr>
    </thead>
    <tbody>
        <AddonRow v-for="entry in props.addons" :key="entry.addon.filename" :entry="entry" />
    </tbody>
</table>
</template>

<script setup lang="ts">
import { AddonEntry } from '../types/Addon.ts';
import AddonRow from './AddonRow.vue';

const props = defineProps<{
    addons: AddonEntry[]
}>()
</script>