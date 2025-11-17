<template>
<div>
    <form @submit.prevent="save">
    <div class="hero is-warning" v-if="hasChanges">
        <div class="container px-6 py-2">
            <div class="level">
                <div class="level-left">
                    <div class="level-item">
                        <h4 class="title is-4 is-inline-block" style="vertical-align: bottom">Unsaved Changes</h4>
                    </div>
                    <div class="level-item">
                        <Button type="submit" color="is-link" class="ml-2" icon-left="iconoir:check">Save</Button>
                        <Button @click="reset" class="ml-2" icon-left="iconoir:undo">Revert</Button>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="container px-6">
        <br>
        <h4 class="title is-4">
            <IconVue class="icon" :inline="true" icon="iconoir:settings" />
            Settings
        </h4>
        <div class="box has-background-info-light">
            <Field label="Addons Folder Path" :error="validationErrors['addonsPath']">
                <input type="text" :class="['input',{'is-danger': validationErrors['addonsPath']}]" v-model="newConfig.addons_folder" :error="validationErrors['addonsPath']" required />
                <p class="help">Path to your addons (example: steam/steamapps/common/Left 4 Dead2/left4dead2/addons)</p>
            </Field>

            <Field label="Steam API Key (optional)" :error="validationErrors['apiKey']">
                <input type="text" :class="['input',{'is-danger': validationErrors['apiKey']}]" 
                    v-model.trim="newConfig.steam_apikey" maxlength="32" minlenth="32" pattern="[A-Za-z0-9]{32}" />
                <p class="help">Allows you to automatically unsubscribe from workshop items. <a target="_blank" href="https://steamcommunity.com/dev/apikey">Get your key here.</a> Requires Steam Authenticator.</p>
            </Field>
        </div>

        <br>

        <h4 class="title is-4">
            <IconVue class="icon" :inline="true" icon="iconoir:switch-off" />
            Preferences
        </h4>
        <div class="box has-background-info-light">
            <!-- TODO: fix weird horizontal alignment of checkboxes. no idea why the same two line up diff -->
            <Field>
                <label class="checkbox large">
                    <input type="checkbox" class="checkbox large" v-model="newConfig.startup_scan">
                    Start a scan on startup
                </label>
            </Field>
            <Field>
                <label class="checkbox large">
                    <input type="checkbox" class="checkbox large" v-model="newConfig.startup_telemetry">
                    Enable Telemetry at startup
                </label>
                <p class="help">This will send OS, OS version, app version, and number of addons to help me understand how this app is being used</p>
            </Field>
        </div>

        <br>

        <h4 class="title is-4">
            <IconVue class="icon" :inline="true" icon="iconoir:warning-triangle" />
            Danger Zone
        </h4>
        <div class="box has-background-danger-light">
            <div class="buttons">
                <button class="button is-danger has-text-weight-bold" @click="promptReset">Reset Database</button>
            </div>
        </div>
        <br>
    </div>
    </form>
</div>
</template>

<script setup lang="ts">
import { computed, onActivated, onBeforeMount, onMounted, ref, watch } from 'vue';
import Field from '../components/Field.vue';
import { AppConfig } from '../types/App';
import { resetDatabase, setConfig } from '../js/tauri.ts';
import { notify } from '@kyvg/vue3-notification';
import { confirm } from '@tauri-apps/plugin-dialog';
import Icon from '../components/Icon.vue';
import { Icon as IconVue } from '@iconify/vue'
import Button from '../components/Button.vue';

const emit = defineEmits(["config-changed"])

const props = defineProps<{
    config: AppConfig
}>()

const newConfig = ref<AppConfig>({ 
    // these values are not the real defaults
    startup_scan: false,
    startup_telemetry: false,
    steam_apikey: null,
    addons_folder: ""
})

const validationErrors = computed(() => {
    const errors: Record<string, string> = {}

    if(newConfig.value.addons_folder?.length === 0) errors['addonsPath'] = "Addons path must be set"
    const keyLen = newConfig.value.steam_apikey?.length
    if(keyLen && keyLen > 0 && keyLen != 32) errors["apiKey"] = "Steam API Key must be 32 characters long"

    return errors
})

const hasChanges = computed(() => {
    for(const [key, val] of Object.entries(props.config)) {
        //@ts-expect-error its the same interface type, don't care about key interfacing crap
        if(val !== newConfig.value[key]) return true
    }
    return false
})

const canSave = computed(() => {
    return Object.keys(validationErrors.value).length === 0 && hasChanges.value
})

async function save() {
    if(!canSave.value) return
    await setConfig(newConfig.value)
    notify({
        type: "success",
        title: "Settings saved successfully",
    })
}

async function promptReset() {
    if(await confirm(
        "This will require a full rescan of all your addons and may take a few minutes. All tags you have added will be lost.", 
        { kind: "warning", title: "Are you sure?", okLabel: "Yes", cancelLabel: "No"}
    )) {
        await resetDatabase()
    }
}

watch(() => props.config, reset)
onMounted(() => reset())
function reset() {
    newConfig.value = Object.assign({}, props.config)
}
</script>