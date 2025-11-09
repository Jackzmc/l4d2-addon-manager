<template>
<div class="container px-6">
    <br>
    <form @submit.prevent="save">
        <Field label="Addons Folder Path" :error="validationErrors['addonsPath']">
            <input type="text" :class="['input',{'is-danger': validationErrors['addonsPath']}]" v-model="addonsPath" :error="validationErrors['addonsPath']"/>
            <p class="help">Path to your addons (example: steam/steamapps/common/Left 4 Dead2/left4dead2/addons)</p>
        </Field>

        <Field label="Steam API Key (optional)" :error="validationErrors['apiKey']">
            <input type="text" :class="['input',{'is-danger': validationErrors['apiKey']}]" v-model="apiKey" />
            <p class="help">Allows you to automatically unsubscribe from workshop items. <a target="_blank" href="https://steamcommunity.com/dev/apikey">Get your key here</a></p>
        </Field>

        <Field>
            <button type="submit" class="button is-link" :disabled="canSave ? undefined : true">Save Changes</button>
        </Field>
    </form>
</div>
</template>

<script setup lang="ts">
import { computed, onActivated, onBeforeMount, onMounted, ref } from 'vue';
import Field from '../components/Field.vue';
import { AppConfig } from '../types/App';
import { setConfig } from '../js/tauri.ts';
import { notify } from '@kyvg/vue3-notification';

const emit = defineEmits(["config-changed"])

const props = defineProps<{
    config: AppConfig
}>()

const addonsPath = ref("")
const apiKey = ref("")

const validationErrors = computed(() => {
    const errors: Record<string, string> = {}

    if(addonsPath.value.length === 0) errors['addonsPath'] = "Addons path must be set"
    if(apiKey.value != "" && apiKey.value.length !== 32) errors["apiKey"] = "Steam API Key must be 32 characters long"

    return errors
})

const hasChanges = computed(() => {
    if(props.config.addons_folder !== addonsPath.value) return true
    if(props.config.steam_apikey !== apiKey.value) return true
    return false
})

const canSave = computed(() => {
    return Object.keys(validationErrors.value).length === 0 && hasChanges.value
})

async function save() {
    if(!canSave.value) return
    const newConfig = {
        addons_folder: addonsPath.value,
        steam_apikey: apiKey.value
    }
    await setConfig(newConfig)
    notify({
        type: "success",
        title: "Settings saved successfully"
    })
}

onBeforeMount(() => {
    addonsPath.value = props.config.addons_folder ?? ""
    apiKey.value = props.config.steam_apikey ?? ""
})
</script>