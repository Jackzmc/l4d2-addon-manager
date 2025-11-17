<template>
    <Component :is="component" :type="type" target="_blank" :href="href" :class="classList" :disabled="disabled ? true : undefined">
        <Icon button inline v-if="iconLeft" :icon="iconLeft" class="mr-2" />
        <slot />
        <Icon button inline v-if="iconRight" :icon="iconRight" class="ml-2" />
    </Component>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Icon from './Icon.vue';

const props = defineProps<{
    loading?: boolean,
    disabled?: boolean,
    size?: "is-large" | "is-medium" | "is-small" | "is-tiny",
    color?: "is-info" | "is-link" | "is-primary" | "is-success" | "is-warning" | "is-danger"
    iconLeft?: string
    iconRight?: string,
    href?: string,
    type?: string
}>()

const classList = computed(() => {
    return ['button', props.size, props.color, {'is-loading': props.loading}]
})

const component = computed(() => {
    if(props.type) return "input"
    if(props.href) return "a"
    return "button"
})
</script>