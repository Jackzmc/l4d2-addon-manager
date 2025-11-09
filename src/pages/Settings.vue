<template>
<div class="container px-6">
    <div class="notification is-warning">
        Not implemented yet
    </div>
    <form>
        <Field label="Addons Folder Path">
            <input type="text" class="input" v-model="addonsPath" @change="onAddonsPathChange" />
            <p class="help">Path to your addons (example: steam/steamapps/common/Left 4 Dead2/left4dead2/addons)</p>
        </Field>

        <Field label="Steam API Key">
            <input type="text" class="input" v-model="apiKey" @change="onAPIKeyChange" />
            <p class="help">Allows you to automatically unsubscribe from workshop items. <a target="_blank" href="https://steamcommunity.com/dev/apikey">Get your key here</a></p>
        </Field>

        <div class="block">
            <button class="button is-link" type="submit">Save</button> 
        </div>
    </form>
</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import Field from '../components/Field.vue';
import { AppConfig } from '../types/App';

const props = defineProps<{
    config: AppConfig
}>()

const addonsPath = ref()
const apiKey = ref()

async function onAddonsPathChange() {

}

async function onAPIKeyChange() {
    await tryInvoke("")
}

onMounted(() => {
    addonsPath.value = props.config.addons_folder
    apiKey.value = props.config.steam_apikey
})
</script>