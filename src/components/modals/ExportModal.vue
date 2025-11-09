<template>
    <ModalCard active title="Export">
        <p class="subtitle">This will export the app's database and config. This includes your steam API key</p>
        <Field label="Include Addons Folder">
            <input type="checkbox" class="input">
            <p class="help">Should the export zip also include a backup of your addons/ folder?</p>
        </Field>
    </ModalCard>
</template>

<script setup lang="ts">
import ModalCard from '../ModalCard.vue';

</script>
