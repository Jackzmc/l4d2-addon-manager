<template>
<div class="container px-6">
    <div class="notification is-warning">
        Not implemented yet
    </div>
    <form>
        <Field label="Addons Folder Path">
            <input type="text" class="input" v-model="addonsPath" />
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

const addonsPath = ref()

onMounted(() => {

})
</script>