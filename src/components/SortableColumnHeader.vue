<template>
<th>
    <Icon @click="onClick" :class="['is-clickable',{'has-text-info': props.sort && props.sort.field === props.field}]" 
        :icon="icon"  style="transform: scale(1.1)" :text-left="label" />
</th>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Icon from './Icon.vue';

const emit = defineEmits(["sort"])

export interface SelectedSort {
    field: string,
    descending: boolean
}

const props = defineProps<{
    label: string,
    field: string,
    sort: SelectedSort | null
}>()

const icon = computed(() => {
    if(!props.sort || props.sort.field !== props.field) {
        return "iconoir:sort"
    }
    return props.sort.descending ? "iconoir:sort-down" : "iconoir:sort-up" 
})

function onClick() {
    if(!props.sort || props.sort.field === props.field) {
        emit("sort", props.field, false)
    } else {
        emit("sort", props.field, !props.sort.descending)
    }
}
</script>