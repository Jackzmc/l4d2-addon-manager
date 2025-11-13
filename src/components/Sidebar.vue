<template>
<nav class="panel is-primary sidebar">
    <p class="panel-heading has-background-info-light l4d2-text">L4D2 Addon Manager <span class="is-size-7 tag">v{{ appData.app_version }}</span></p>
    <router-link class="panel-block is-block" :to="{ name: 'addons-manual' }">
        Managed Addons
        <span class="tag is-rounded is-pulled-right is-dark px-3 is-family-code">{{ counts.addons }}</span>
    </router-link>
    <router-link class="panel-block is-block" :to="{ name: 'addons-workshop' }">
        Workshop Addons
        <span class="tag is-rounded is-pulled-right is-dark px-3 is-family-code">{{ counts.workshop }}</span>
    </router-link>
    <div class="panel-block">
        <br>
    </div>
    <a class="panel-block" @click="emit('scan')">
        <Icon v-if="props.scanActive"icon="xmark-circle" class="has-text-danger has-tooltip-right has-tooltip-danger" 
            data-tooltip="Cancel the currently running scan (may take a few moments)">
            Cancel Scan
        </Icon>
        <Icon v-else icon="page-search" >Start Scan</Icon> 
    </a>
    <router-link class="panel-block" :to="{ name: 'settings' }">
        <Icon icon="settings">Settings</Icon> 
    </router-link>
    <router-link class="panel-block" :to="{ name: 'export'}">
        <Icon icon="database-export">Export</Icon>
    </router-link>
    <router-link class="panel-block" :to="{ name: 'logs'}">
        <Icon icon="terminal">Logs</Icon>
    </router-link>
    <!-- <a class="panel-block" @click="checkForUpdates">
        Check for Updates
    </a> -->

    <footer v-if="appData">
        v{{ appData.app_version }}
        <template v-if="appData.git_commit">
            git {{ appData.git_commit }}
        </template>
        {{ size[0] }}x{{ size[1] }}
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
import { AddonCounts, StaticAppData } from '../types/App.ts';
import { check } from '@tauri-apps/plugin-updater';
import { notify } from '@kyvg/vue3-notification';
import { onMounted, ref } from 'vue';
import Icon from './Icon.vue';

const props = defineProps<{
    scanActive: boolean,
    appData: StaticAppData,
    counts: AddonCounts
}>()
const emit = defineEmits(["scan"])

const size = ref([0, 0])

async function checkForUpdates() {
    const update = await check();
    if(!update) return notify({
        type: "info",
        title: "No update found"
    })
    console.info(
        `found update ${update.version} from ${update.date} with notes ${update.body}`
    );
    notify({
        type: "info",
        title: "Update Found",
        text: "Downloading update"
    })
    let downloaded = 0;
    let contentLength = 0;
    // alternatively we could also call update.download() and update.install() separately
    await update.downloadAndInstall((event) => {
        switch (event.event) {
            case 'Started':
                contentLength = event.data.contentLength ?? 0;
                console.log(`started downloading ${event.data.contentLength} bytes`);
                break;
            case 'Progress':
                downloaded += event.data.chunkLength;
                console.log(`downloaded ${downloaded} from ${contentLength}`);
                break;
            case 'Finished':
                console.log('download finished');
                break;
        }
    });

    console.log('update installed');
    notify({
        type: "info",
        title: "Update Complete",
        text: "Restart app to launch updated version"
    })
}

onMounted(() => {
    size.value = [window.outerWidth, window.outerHeight]
    addEventListener("resize", (_) => {
        size.value = [window.outerWidth, window.outerHeight]
    })
})
</script>