<template>
<nav class="panel is-primary sidebar">
    <p class="panel-heading">L4D2 Addon Manager</p>
    <router-link class="panel-block" :to="{ name: 'addons-manual' }">
        Managed Addons
    </router-link>
    <router-link class="panel-block" :to="{ name: 'addons-workshop' }">
        Workshop Addons
    </router-link>
    <div class="panel-block">
        <br>
    </div>
    <a class="panel-block" @click="emit('scan')">
        {{ props.scanActive ? 'Cancel Scan' : 'Start Scan' }}
    </a>
    <router-link class="panel-block" :to="{ name: 'settings' }">
        Settings
    </router-link>

    <footer>
        v{{ appVersion }}
    </footer>
</nav>
</template>

<style scoped>
.sidebar, .sidebar .panel-heading, .sidebar .panel-block {
    border-radius: 0;
}
.sidebar {
    height: 100%;
}
.sidebar .panel-block {
    padding-left: 1em;
}
.sidebar .panel-block.is-active {
    background-color: lightblue;
    font-weight: bold;
    color: black;
}
.sidebar {
    border: 1px solid lightgray;
}
.sidebar footer {
    position: absolute;
    bottom: 0;
    padding-left: 5px;
}
</style>

<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app';
import { onBeforeMount, ref } from 'vue';

const appVersion = ref("")

const props = defineProps<{
    scanActive: boolean
}>()
const emit = defineEmits(["scan"])

onBeforeMount(async () => {
    appVersion.value = await getVersion();
})
</script>