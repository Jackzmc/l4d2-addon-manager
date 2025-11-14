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
    <a v-if="scanState === ScanState.Running" class="panel-block has-text-danger has-tooltip-right has-tooltip-danger" @click="emit('scan')" data-tooltip="Cancel the currently running scan (may take a few moments)">
        <Icon icon="xmark-circle">Cancel Scan</Icon>
    </a>
    <a v-else-if="scanState === ScanState.Cancelling" class="panel-block has-text-warning" style="cursor: not-allowed">
        <Icon icon="hourglass">Scan is stopping</Icon>
    </a>
    <a v-else class="panel-block" @click="emit('scan')">
        <Icon icon="page-search">Start Scan</Icon> 
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
    <router-link class="panel-block" :to="{ name: 'about'}">
        <Icon icon="info-circle">About</Icon>
    </router-link>
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
import { ScanState } from '../types/Scan.ts';

const props = defineProps<{
    scanState: ScanState,
    appData: StaticAppData,
    counts: AddonCounts
}>()
const emit = defineEmits(["scan"])

const size = ref([0, 0])
const isCancelling = ref(false)

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