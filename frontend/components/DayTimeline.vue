<template>
    <div class="wrapper">
        <TimelineEvent v-for="event in $props.events" :event="event" />
    </div>
</template>

<script setup lang="ts">
import { CalendarEvent } from "@/utils/CalendarEvent";

const props = defineProps<{
    events: CalendarEvent[];
}>();
</script>

<style scoped>
.wrapper {
    position: relative;
}
</style>
